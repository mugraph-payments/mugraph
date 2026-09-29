use std::collections::{BTreeMap, HashSet};

#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
use blake2::Digest;
use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    keyset,
    types::{
        BlindSignature, BlindedOutput, Keypair, Note, Response,
        WithdrawRequest, WithdrawalStatus,
    },
};
#[cfg(test)]
use whisky_csl::csl;

#[cfg(test)]
use crate::tx_signer::compute_tx_hash;
use crate::{
    database::WITHDRAWALS, routes::Context,
    tx_signer::attach_witness_to_transaction,
};

mod input_validation;
mod io;
mod parsed_tx;
mod state;
mod tx_checks;

pub(super) use self::parsed_tx::ParsedWithdrawalTx;
#[cfg(test)]
use self::{
    input_validation::checked_output_index,
    tx_checks::{
        validate_network_and_vault_outputs, validate_transaction_balance,
        validate_transaction_balance_with_tolerance,
    },
};
use self::{
    input_validation::{
        validate_script_inputs_with_parsed_tx, validate_signers_with_parsed_tx,
    },
    io::{create_provider, load_wallet, submit_transaction},
    state::{
        atomic_burn_and_record_pending, mark_withdrawal_completed,
        mark_withdrawal_failed,
    },
    tx_checks::{
        validate_network_and_vault_outputs_with_parsed_tx, validate_parsed_fee,
        validate_transaction_balance_with_parsed_tx, validate_withdraw_value,
    },
};

/// Handle withdrawal request
///
/// 1. Parse the transaction and check idempotency, size and fee
/// 2. Verify each burned note, and each change output
/// 3. Ensure all inputs are vault UTxOs for this node
/// 4. Check the witnesses and that the node is a required signer
/// 5. Check the ledger balance, the network, and the vault outputs
/// 6. Check that notes - change = the value that leaves the vault
/// 7. Attach the node witness, burn the notes, and submit
/// 8. Return signed CBOR + hash + change notes
pub async fn handle_withdraw(
    request: &WithdrawRequest,
    ctx: &Context,
) -> Result<Response, Error> {
    tracing::info!(
        "Processing withdrawal request for tx_hash: {}",
        &request.tx_hash[..std::cmp::min(16, request.tx_hash.len())]
    );

    // 1. Preflight validation
    let provider = create_provider(ctx)?;
    let parsed_tx = ParsedWithdrawalTx::parse(&request.tx_cbor)?;

    check_idempotency(request, ctx)?;

    if parsed_tx.tx_cbor.len() > ctx.config.max_tx_size() {
        return Err(Error::InvalidInput {
            reason: format!(
                "Transaction size {} bytes exceeds maximum {} bytes",
                parsed_tx.tx_cbor.len(),
                ctx.config.max_tx_size()
            ),
        });
    }

    let _fee = validate_parsed_fee(
        &parsed_tx,
        ctx.config.max_withdrawal_fee(),
        ctx.config.fee_tolerance_pct(),
    )?;

    let wallet = load_wallet(ctx)?;

    let computed_hash = parsed_tx.tx_hash_hex.clone();
    if computed_hash != request.tx_hash {
        return Err(Error::InvalidInput {
            reason: format!(
                "Transaction hash mismatch: computed {}, provided {}",
                computed_hash, request.tx_hash
            ),
        });
    }

    // 2. The notes are the claim on the vault, so check them first.
    let note_totals = validate_notes(&request.notes, &ctx.keypair)?;
    let change_totals = validate_change_outputs(&request.change_outputs)?;

    // 3. Inputs must be vault UTxOs for this node
    let (input_totals, consumed_deposits) =
        validate_script_inputs_with_parsed_tx(&parsed_tx, &wallet, &provider)
            .await?;

    // 4. Witnesses and required signers
    validate_signers_with_parsed_tx(&parsed_tx, &wallet)?;

    // 5. Ledger balance, network, and vault outputs
    validate_transaction_balance_with_parsed_tx(
        &parsed_tx,
        &input_totals,
        ctx.config.max_withdrawal_fee(),
        ctx.config.fee_tolerance_pct(),
    )?;
    let vault_out =
        validate_network_and_vault_outputs_with_parsed_tx(&parsed_tx, &wallet)?;

    // 6. The notes pay for exactly the value that leaves the vault
    let vault_in: BTreeMap<String, u128> = input_totals.into_iter().collect();
    validate_withdraw_value(
        &vault_in,
        &vault_out,
        &note_totals,
        &change_totals,
    )?;

    // 7. Sign, burn, and submit
    let signed_cbor = attach_witness_to_transaction(
        &parsed_tx.tx_cbor,
        &parsed_tx.tx_hash,
        &wallet,
    )
    .map_err(|e| Error::Internal {
        reason: format!("Failed to sign transaction: {}", e),
    })?;
    let signed_cbor_hex = hex::encode(&signed_cbor);

    // Calculate change notes before any state changes
    let change_notes = calculate_change_notes(request, &ctx.keypair)?;

    // Update state atomically BEFORE submitting to provider
    // This ensures we only submit if we can properly track the withdrawal
    let pending_tx_hash = request.tx_hash.clone();
    match atomic_burn_and_record_pending(request, ctx, &pending_tx_hash) {
        Ok(()) => {
            tracing::info!("Notes burned and withdrawal recorded as pending");
        }
        Err(e) => {
            tracing::error!("Failed to prepare withdrawal state: {}", e);
            return Err(e);
        }
    }

    let submit_response = match submit_transaction(&signed_cbor_hex, &provider)
        .await
    {
        Ok(response) => response,
        Err(e) => {
            // Submission can fail ambiguously (e.g. timeout after relay acceptance).
            // Keep burned/pending state for deterministic reconciliation instead of unburning.
            tracing::error!(
                "Transaction submission failed after notes were burned: {}. Marking withdrawal failed for recovery.",
                e
            );
            if let Err(mark_err) = mark_withdrawal_failed(ctx, &pending_tx_hash)
            {
                tracing::error!(
                    "Failed to mark withdrawal as failed after submit error: {} (tx {})",
                    mark_err,
                    pending_tx_hash
                );
            }

            return Err(Error::NetworkError {
                reason: format!("Transaction submission failed: {}", e),
            });
        }
    };

    if submit_response.tx_hash != pending_tx_hash {
        tracing::error!(
            "Provider returned mismatched tx hash: expected {}, got {}",
            pending_tx_hash,
            submit_response.tx_hash
        );
        mark_withdrawal_failed(ctx, &pending_tx_hash)?;
        return Err(Error::Internal {
            reason: format!(
                "Provider returned mismatched tx hash: expected {}, got {}",
                pending_tx_hash, submit_response.tx_hash
            ),
        });
    }

    let mark_result =
        mark_withdrawal_completed(ctx, &pending_tx_hash, &consumed_deposits);

    finalize_withdraw_response(
        mark_result,
        signed_cbor_hex,
        pending_tx_hash,
        change_notes,
    )
}

/// Verifies each note that the withdrawal burns, and returns their total
/// value for each unit.
///
/// A note is valid only if this node signed it with the key for its asset
/// and amount. The spent check happens later, in the same database
/// transaction that burns the notes.
fn validate_notes(
    notes: &[Note],
    keypair: &Keypair,
) -> Result<BTreeMap<String, u128>, Error> {
    if notes.is_empty() {
        return Err(Error::InvalidInput {
            reason: "No notes to burn".to_string(),
        });
    }

    let mut seen = HashSet::new();
    let mut totals = BTreeMap::new();

    for (i, note) in notes.iter().enumerate() {
        if note.delegate != keypair.public_key {
            return Err(Error::InvalidInput {
                reason: format!("Note {} is for a different delegate", i),
            });
        }

        if !seen.insert(note.signature) {
            return Err(Error::InvalidInput {
                reason: format!("Note {} is in the request more than once", i),
            });
        }

        let asset = mugraph_core::types::Asset {
            policy_id: note.policy_id,
            asset_name: note.asset_name,
        };
        if !keyset::verify(
            &keypair.secret_key,
            &asset,
            note.amount,
            note.commitment().as_ref(),
            note.signature,
        )? {
            return Err(Error::InvalidSignature {
                reason: format!("Note {} has an invalid signature", i),
                signature: note.signature,
            });
        }

        *totals.entry(asset.cardano_unit()).or_default() += note.amount as u128;
    }

    Ok(totals)
}

/// Checks that each change output is one denomination, and returns their
/// total value for each unit.
fn validate_change_outputs(
    outputs: &[BlindedOutput],
) -> Result<BTreeMap<String, u128>, Error> {
    let mut totals = BTreeMap::new();

    for (i, output) in outputs.iter().enumerate() {
        if !keyset::is_denomination(output.amount) {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Change output {} amount {} is not a power of two",
                    i, output.amount
                ),
            });
        }
        *totals.entry(output.asset().cardano_unit()).or_default() +=
            output.amount as u128;
    }

    Ok(totals)
}

fn finalize_withdraw_response(
    mark_result: Result<(), Error>,
    signed_tx_cbor: String,
    tx_hash: String,
    change_notes: Vec<BlindSignature>,
) -> Result<Response, Error> {
    match mark_result {
        Ok(()) => {
            tracing::info!(
                "Withdrawal completed successfully: {}",
                &tx_hash[..std::cmp::min(16, tx_hash.len())]
            );

            Ok(Response::Withdraw {
                signed_tx_cbor,
                tx_hash,
                change_notes,
            })
        }
        Err(e) => {
            tracing::error!(
                "CRITICAL: Transaction {} was submitted but marking as completed failed: {}.",
                tx_hash,
                e
            );

            Err(Error::Internal {
                reason: format!(
                    "Transaction {} submitted but completion state update failed: {}",
                    tx_hash, e
                ),
            })
        }
    }
}

/// Check if withdrawal has already been processed (idempotency)
fn check_idempotency(
    request: &WithdrawRequest,
    ctx: &Context,
) -> Result<(), Error> {
    let read_tx = ctx.database.read()?;
    let table = read_tx.open_table(WITHDRAWALS)?;

    // Use network byte from config
    let network_byte = ctx.config.network_byte();
    let key =
        crate::tx_ids::parse_withdrawal_key(&request.tx_hash, network_byte)?;

    if let Some(record) = table.get(key)? {
        let record = record.value();
        // Check if already completed (pending or failed can be retried)
        if record.status == WithdrawalStatus::Completed {
            return Err(Error::InvalidInput {
                reason: "Withdrawal already completed".to_string(),
            });
        }
        // Log warning if retrying a failed withdrawal
        if record.status == WithdrawalStatus::Failed {
            tracing::warn!(
                "Retrying previously failed withdrawal for tx {}",
                request.tx_hash
            );
        }
    }

    Ok(())
}

/// Signs each change output with the key for its asset and amount.
fn calculate_change_notes(
    request: &WithdrawRequest,
    keypair: &Keypair,
) -> Result<Vec<BlindSignature>, Error> {
    let mut rng = rand::rng();
    let mut change_notes = Vec::with_capacity(request.change_outputs.len());

    for output in &request.change_outputs {
        change_notes.push(keyset::sign_blinded(
            &mut rng,
            &keypair.secret_key,
            &output.asset(),
            output.amount,
            &output.point.to_point()?,
        )?);
    }

    Ok(change_notes)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        Router,
        extract::Path,
        http::StatusCode,
        response::IntoResponse,
        routing::{get, post},
    };
    use ed25519_dalek::SigningKey;
    use mugraph_core::{
        keyset,
        types::{Asset, BlindedOutput, Note, PendingNote, blind_new_note},
    };
    use rand::{SeedableRng, rngs::StdRng};
    use serde_json::json;
    use tempfile::TempDir;

    use super::*;
    use crate::{
        cardano::generate_payment_keypair,
        config::Config,
        database::{CARDANO_WALLET, DEPOSITS, Database, NOTES, WITHDRAWALS},
        routes::Context,
    };

    const USER_ADDRESS: &str =
        "addr_test1vru4e2un2tq50q4rv6qzk7t8w34gjdtw3y2uzuqxzj0ldrqqactxh";

    fn test_context() -> Context {
        test_context_with_provider_url(None)
    }

    fn test_context_with_provider_url(provider_url: Option<String>) -> Context {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("db.redb");
        let database = Arc::new(Database::setup(db_path).unwrap());
        database.migrate().unwrap();
        std::mem::forget(dir);

        let config = Config::Server {
            addr: "127.0.0.1:9999".parse().unwrap(),
            seed: Some(7),
            secret_key: None,
            cardano_network: "preprod".to_string(),
            cardano_provider: "blockfrost".to_string(),
            cardano_api_key: Some("test".to_string()),
            cardano_provider_url: provider_url,
            cardano_payment_sk: None,
            xnode_peer_registry_file: None,
            xnode_node_id: "node://local".to_string(),
            deposit_confirm_depth: 15,
            deposit_expiration_blocks: 1440,
            min_deposit_value: Some(1_000_000),
            max_tx_size: 16384,
            max_withdrawal_fee: 2_000_000,
            fee_tolerance_pct: 5,
            dev_mode: true,
        };
        let keypair = config.keypair().unwrap();

        Context {
            keypair,
            database,
            config,
            peer_registry: None,
        }
    }

    /// A valid testnet enterprise address, used as the vault address.
    fn vault_address() -> String {
        let key_hash = csl::Ed25519KeyHash::from_bytes(vec![9u8; 28]).unwrap();
        let cred = csl::Credential::from_keyhash(&key_hash);
        csl::EnterpriseAddress::new(0, &cred)
            .to_address()
            .to_bech32(None)
            .unwrap()
    }

    fn insert_wallet(
        ctx: &Context,
        payment_sk: Vec<u8>,
        payment_vk: Vec<u8>,
        script_address: &str,
    ) {
        let write_tx = ctx.database.write().unwrap();
        {
            let mut table = write_tx.open_table(CARDANO_WALLET).unwrap();
            table
                .insert(
                    "wallet",
                    &mugraph_core::types::CardanoWallet::new(
                        payment_sk,
                        payment_vk,
                        vec![],
                        vec![],
                        script_address.to_string(),
                        "preprod".to_string(),
                    ),
                )
                .unwrap();
        }
        write_tx.commit().unwrap();
    }

    fn seed_deposit(
        ctx: &Context,
        utxo_ref: mugraph_core::types::UtxoRef,
        intent_hash: [u8; 32],
    ) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let record = mugraph_core::types::DepositRecord::with_intent_hash(
            90,
            now,
            now + 3600,
            intent_hash,
        );
        let write_tx = ctx.database.write().unwrap();
        {
            let mut table = write_tx.open_table(DEPOSITS).unwrap();
            table.insert(&utxo_ref, &record).unwrap();
        }
        write_tx.commit().unwrap();
    }

    fn seed_withdrawal_record(
        ctx: &Context,
        tx_hash: &str,
        record: mugraph_core::types::WithdrawalRecord,
    ) {
        let write_tx = ctx.database.write().unwrap();
        {
            let mut table = write_tx.open_table(WITHDRAWALS).unwrap();
            table
                .insert(&withdrawal_key_from_hex(tx_hash), &record)
                .unwrap();
        }
        write_tx.commit().unwrap();
    }

    fn withdrawal_key_from_hex(
        tx_hash: &str,
    ) -> mugraph_core::types::WithdrawalKey {
        let bytes = hex::decode(tx_hash).unwrap();
        let array: [u8; 32] = bytes.try_into().unwrap();
        mugraph_core::types::WithdrawalKey::new(0, array)
    }

    fn node_hash(payment_vk: &[u8]) -> Vec<u8> {
        csl::PublicKey::from_bytes(payment_vk)
            .unwrap()
            .hash()
            .to_bytes()
    }

    /// The vault datum: (user_pubkey_hash, node_pubkey_hash, intent_hash).
    fn vault_datum(node_hash: &[u8]) -> csl::PlutusData {
        let mut fields = csl::PlutusList::new();
        fields.add(&csl::PlutusData::new_bytes(vec![1u8; 28]));
        fields.add(&csl::PlutusData::new_bytes(node_hash.to_vec()));
        fields.add(&csl::PlutusData::new_bytes(vec![0u8; 32]));
        csl::PlutusData::new_constr_plutus_data(&csl::ConstrPlutusData::new(
            &csl::BigNum::zero(),
            &fields,
        ))
    }

    fn vault_datum_hex(node_hash: &[u8]) -> String {
        hex::encode(vault_datum(node_hash).to_bytes())
    }

    /// One output of a test transaction.
    struct Out {
        address: String,
        lovelace: u64,
        /// The datum node hash for a vault output, if any.
        datum_node_hash: Option<Vec<u8>>,
    }

    fn to_user(lovelace: u64) -> Out {
        Out {
            address: USER_ADDRESS.to_string(),
            lovelace,
            datum_node_hash: None,
        }
    }

    fn to_vault(lovelace: u64, node_hash: &[u8]) -> Out {
        Out {
            address: vault_address(),
            lovelace,
            datum_node_hash: Some(node_hash.to_vec()),
        }
    }

    /// Builds a withdrawal transaction that spends one vault UTxO.
    fn build_tx(
        input_tx_hash: [u8; 32],
        outputs: &[Out],
        fee: u64,
        required_signers: &[Vec<u8>],
        signers: &[&SigningKey],
    ) -> (Vec<u8>, String) {
        let tx_hash =
            csl::TransactionHash::from_bytes(input_tx_hash.to_vec()).unwrap();
        let mut inputs = csl::TransactionInputs::new();
        inputs.add(&csl::TransactionInput::new(&tx_hash, 0));

        let mut tx_outputs = csl::TransactionOutputs::new();
        for out in outputs {
            let addr = csl::Address::from_bech32(&out.address).unwrap();
            let value = csl::Value::new(
                &csl::Coin::from_str(&out.lovelace.to_string()).unwrap(),
            );
            let mut output = csl::TransactionOutput::new(&addr, &value);
            if let Some(hash) = &out.datum_node_hash {
                output.set_plutus_data(&vault_datum(hash));
            }
            tx_outputs.add(&output);
        }

        let fee = csl::Coin::from_str(&fee.to_string()).unwrap();
        let mut body =
            csl::TransactionBody::new_tx_body(&inputs, &tx_outputs, &fee);
        if !required_signers.is_empty() {
            let mut required = csl::Ed25519KeyHashes::new();
            for hash in required_signers {
                required.add(
                    &csl::Ed25519KeyHash::from_bytes(hash.clone()).unwrap(),
                );
            }
            body.set_required_signers(&required);
        }

        let body_hash = tx_hash_from_body(&body);
        let mut witness_set = csl::TransactionWitnessSet::new();
        if !signers.is_empty() {
            let mut vkeys = csl::Vkeywitnesses::new();
            for signer in signers {
                let private =
                    csl::PrivateKey::from_normal_bytes(signer.as_bytes())
                        .unwrap();
                vkeys.add(&csl::make_vkey_witness(&body_hash, &private));
            }
            witness_set.set_vkeys(&vkeys);
        }

        let tx = csl::Transaction::new(&body, &witness_set, None);
        let tx_cbor = tx.to_bytes();
        let tx_hash = hex::encode(compute_tx_hash(&tx_cbor).unwrap());
        (tx_cbor, tx_hash)
    }

    /// Notes for `amount` lovelace, one for each denomination.
    fn issue_notes(ctx: &Context, amount: u64) -> Vec<Note> {
        let mut rng = StdRng::seed_from_u64(amount);
        keyset::split_amount(amount)
            .into_iter()
            .map(|part| {
                keyset::issue_note(
                    &mut rng,
                    &ctx.keypair.secret_key,
                    &Asset::default(),
                    part,
                )
                .unwrap()
            })
            .collect()
    }

    fn blinded_change(
        ctx: &Context,
        amount: u64,
    ) -> (Vec<BlindedOutput>, Vec<PendingNote>) {
        let mut rng = StdRng::seed_from_u64(amount + 1);
        keyset::split_amount(amount)
            .into_iter()
            .map(|part| {
                blind_new_note(
                    &mut rng,
                    ctx.keypair.public_key,
                    &Asset::default(),
                    part,
                )
            })
            .unzip()
    }

    fn request(
        tx: (Vec<u8>, String),
        notes: Vec<Note>,
        change_outputs: Vec<BlindedOutput>,
    ) -> WithdrawRequest {
        WithdrawRequest {
            notes,
            change_outputs,
            tx_cbor: hex::encode(tx.0),
            tx_hash: tx.1,
        }
    }

    /// A withdrawal of 1 ADA from a 1.17 ADA vault UTxO, with a 0.17 ADA
    /// fee. The notes pay for the payout and the fee.
    struct Scenario {
        ctx: Context,
        request: WithdrawRequest,
        input_tx_hash: [u8; 32],
    }

    async fn scenario(
        input_tx_hash: [u8; 32],
        submit_status: StatusCode,
        submit_hash: Option<String>,
    ) -> Scenario {
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let hash = node_hash(&payment_vk);
        let tx = build_tx(
            input_tx_hash,
            &[to_user(1_000_000)],
            170_000,
            std::slice::from_ref(&hash),
            &[],
        );
        let tx_hash = tx.1.clone();

        let provider_url = spawn_withdraw_provider_mock(
            vault_address(),
            vault_datum_hex(&hash),
            1_170_000,
            submit_status,
            submit_hash.unwrap_or_else(|| tx_hash.clone()),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, &vault_address());

        let notes = issue_notes(&ctx, 1_170_000);
        Scenario {
            request: request(tx, notes, vec![]),
            ctx,
            input_tx_hash,
        }
    }

    fn note_is_burned(ctx: &Context, note: &Note) -> bool {
        let read_tx = ctx.database.read().unwrap();
        let notes = read_tx.open_table(NOTES).unwrap();
        notes.get(note.signature).unwrap().is_some()
    }

    fn assert_preflight_rejection_leaves_state_untouched(
        ctx: &Context,
        request: &WithdrawRequest,
    ) {
        for note in &request.notes {
            assert!(!note_is_burned(ctx, note), "note must not be burned");
        }
        let read_tx = ctx.database.read().unwrap();
        let withdrawals = read_tx.open_table(WITHDRAWALS).unwrap();
        if let Ok(bytes) = hex::decode(&request.tx_hash)
            && bytes.len() == 32
        {
            let key = withdrawal_key_from_hex(&request.tx_hash);
            assert!(withdrawals.get(&key).unwrap().is_none());
        }
    }

    async fn spawn_withdraw_provider_mock(
        script_address: String,
        datum_cbor_hex: String,
        input_value: u64,
        submit_status: StatusCode,
        submit_hash: String,
    ) -> String {
        type MockState = (String, String, u64, StatusCode, String);

        async fn tx_info() -> impl IntoResponse {
            (StatusCode::OK, axum::Json(json!({"block_height": 90})))
        }

        async fn tx_utxos(
            Path(tx_hash): Path<String>,
            axum::extract::State(state): axum::extract::State<MockState>,
        ) -> impl IntoResponse {
            let (script_address, _, input_value, _, _) = state;
            (
                StatusCode::OK,
                axum::Json(json!({
                    "hash": tx_hash,
                    "outputs": [{
                        "output_index": 0,
                        "address": script_address,
                        "amount": [{"unit": "lovelace", "quantity": input_value.to_string()}],
                        "data_hash": "datumhash",
                        "reference_script_hash": null
                    }]
                })),
            )
        }

        async fn datum_cbor(
            axum::extract::State(state): axum::extract::State<MockState>,
        ) -> impl IntoResponse {
            let (_, datum_hex, _, _, _) = state;
            (StatusCode::OK, axum::Json(json!({"cbor": datum_hex})))
        }

        async fn submit(
            axum::extract::State(state): axum::extract::State<MockState>,
        ) -> impl IntoResponse {
            let (_, _, _, submit_status, submit_hash) = state;
            if submit_status.is_success() {
                (submit_status, axum::Json(json!(submit_hash))).into_response()
            } else {
                (submit_status, "submit failed").into_response()
            }
        }

        let app = Router::new()
            .route("/txs/{tx_hash}", get(tx_info))
            .route("/txs/{tx_hash}/utxos", get(tx_utxos))
            .route("/scripts/datum/{datum_hash}/cbor", get(datum_cbor))
            .route("/tx/submit", post(submit))
            .with_state((
                script_address,
                datum_cbor_hex,
                input_value,
                submit_status,
                submit_hash,
            ));

        let listener =
            tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        format!("http://{addr}")
    }

    async fn spawn_withdraw_provider_mock_without_inline_datum(
        script_address: String,
        input_value: u64,
    ) -> String {
        async fn tx_info() -> impl IntoResponse {
            (StatusCode::OK, axum::Json(json!({"block_height": 90})))
        }

        async fn tx_utxos(
            Path(tx_hash): Path<String>,
            axum::extract::State(state): axum::extract::State<(String, u64)>,
        ) -> impl IntoResponse {
            let (script_address, input_value) = state;
            (
                StatusCode::OK,
                axum::Json(json!({
                    "hash": tx_hash,
                    "outputs": [{
                        "output_index": 0,
                        "address": script_address,
                        "amount": [{"unit": "lovelace", "quantity": input_value.to_string()}],
                        "data_hash": null,
                        "reference_script_hash": null
                    }]
                })),
            )
        }

        let app = Router::new()
            .route("/txs/{tx_hash}", get(tx_info))
            .route("/txs/{tx_hash}/utxos", get(tx_utxos))
            .with_state((script_address, input_value));

        let listener =
            tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        format!("http://{addr}")
    }

    async fn spawn_withdraw_provider_mock_with_utxo_failure() -> String {
        async fn tx_utxos() -> impl IntoResponse {
            (StatusCode::INTERNAL_SERVER_ERROR, "boom")
        }

        let app = Router::new().route("/txs/{tx_hash}/utxos", get(tx_utxos));

        let listener =
            tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        format!("http://{addr}")
    }

    // --- Pure checks ------------------------------------------------------

    #[test]
    fn rejects_input_index_overflow() {
        let err = checked_output_index(u16::MAX as u32 + 1, 0).unwrap_err();
        assert!(format!("{err:?}").contains("exceeds u16::MAX"));
    }

    #[test]
    fn test_validate_transaction_balance() {
        let tx = minimal_tx_with_values(1_000_000, 1_000_000); // output 1ADA, fee 1ADA
        let tx_cbor = tx.to_bytes();
        let mut totals = HashMap::new();
        totals.insert("lovelace".to_string(), 2_000_000u128);
        let max_fee = 1_100_000;

        assert!(
            validate_transaction_balance(&tx_cbor, &totals, max_fee).is_ok()
        );

        // Fee too high
        let max_fee = 500_000;
        assert!(
            validate_transaction_balance(&tx_cbor, &totals, max_fee).is_err()
        );
    }

    #[test]
    fn test_validate_transaction_balance_respects_fee_tolerance() {
        let tx = minimal_tx_with_values(1_000_000, 1_050_000); // fee within 5% of 1_000_000
        let tx_cbor = tx.to_bytes();
        let mut totals = HashMap::new();
        totals.insert("lovelace".to_string(), 2_050_000u128);

        let res = validate_transaction_balance_with_tolerance(
            &tx_cbor, &totals, 1_000_000, 5,
        );
        assert!(res.is_ok());
    }

    #[test]
    fn test_multiasset_imbalance_rejected() {
        // Inputs: 1 ADA + 5 tokens; Outputs: 1 ADA + 6 tokens -> should fail
        let policy_hex = "00".repeat(28); // 28-byte script hash in hex
        let asset_hex = "746f6b656e"; // "token"
        let tx = tx_with_multiasset_output(
            1_000_000,
            &[(&policy_hex, asset_hex, 6)],
        );
        let tx_cbor = tx.to_bytes();
        let mut inputs = HashMap::new();
        inputs.insert("lovelace".to_string(), 1_000_000u128);
        inputs.insert(format!("{}{}", policy_hex, asset_hex), 5u128);
        let res = validate_transaction_balance(&tx_cbor, &inputs, 200_000);
        assert!(res.is_err());
    }

    #[test]
    fn test_multiasset_balance_accepted() {
        // Inputs: 1 ADA + 5 tokens; Outputs: 1 ADA + 5 tokens -> balanced.
        // Units use the raw asset name bytes, as the provider shows them.
        let policy_hex = "00".repeat(28);
        let asset_hex = "746f6b656e";
        let tx = tx_with_multiasset_output(
            1_000_000,
            &[(&policy_hex, asset_hex, 5)],
        );
        let mut inputs = HashMap::new();
        inputs.insert("lovelace".to_string(), 1_000_000u128);
        inputs.insert(format!("{}{}", policy_hex, asset_hex), 5u128);

        validate_transaction_balance(&tx.to_bytes(), &inputs, 200_000)
            .expect("a balanced multi-asset transaction must pass");
    }

    #[test]
    fn test_multiasset_phantom_asset_rejected() {
        // Inputs: only ADA; Outputs: ADA + new token -> should fail
        let policy_hex = "00".repeat(28);
        let asset_hex = "746f6b656e";
        let tx = tx_with_multiasset_output(
            1_000_000,
            &[(&policy_hex, asset_hex, 1)],
        );
        let tx_cbor = tx.to_bytes();
        let mut inputs = HashMap::new();
        inputs.insert("lovelace".to_string(), 1_100_000u128); // cover fee + output
        let res = validate_transaction_balance(&tx_cbor, &inputs, 200_000);
        assert!(res.is_err());
    }

    fn totals(pairs: &[(&str, u128)]) -> BTreeMap<String, u128> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn withdraw_value_accepts_notes_equal_to_the_vault_outflow() {
        validate_withdraw_value(
            &totals(&[("lovelace", 1_170_000)]),
            &totals(&[("lovelace", 100_000)]),
            &totals(&[("lovelace", 1_170_000)]),
            &totals(&[("lovelace", 100_000)]),
        )
        .expect("notes - change == vault in - vault out");
    }

    #[test]
    fn withdraw_value_rejects_notes_below_the_vault_outflow() {
        let err = validate_withdraw_value(
            &totals(&[("lovelace", 1_170_000)]),
            &totals(&[]),
            &totals(&[("lovelace", 1)]),
            &totals(&[]),
        )
        .unwrap_err();
        assert!(format!("{err:?}").contains("does not match"), "{err:?}");
    }

    #[test]
    fn withdraw_value_rejects_notes_above_the_vault_outflow() {
        let err = validate_withdraw_value(
            &totals(&[("lovelace", 1_000_000)]),
            &totals(&[]),
            &totals(&[("lovelace", 1_000_001)]),
            &totals(&[]),
        )
        .unwrap_err();
        assert!(format!("{err:?}").contains("does not match"), "{err:?}");
    }

    #[test]
    fn withdraw_value_rejects_tokens_that_leave_without_notes() {
        let token = format!("{}{}", "11".repeat(28), "746f6b656e");
        let err = validate_withdraw_value(
            &totals(&[("lovelace", 1_000_000), (&token, 5)]),
            &totals(&[]),
            &totals(&[("lovelace", 1_000_000)]),
            &totals(&[]),
        )
        .unwrap_err();
        assert!(format!("{err:?}").contains("does not match"), "{err:?}");
    }

    #[test]
    fn validate_notes_sums_valid_notes_by_asset() {
        let ctx = test_context();
        let notes = issue_notes(&ctx, 1_170_000);

        let sums = validate_notes(&notes, &ctx.keypair).unwrap();
        assert_eq!(sums, totals(&[("lovelace", 1_170_000)]));
    }

    #[test]
    fn validate_notes_rejects_an_empty_list() {
        let ctx = test_context();
        let err = validate_notes(&[], &ctx.keypair).unwrap_err();
        assert!(format!("{err:?}").contains("No notes"), "{err:?}");
    }

    #[test]
    fn validate_notes_rejects_a_note_from_another_key() {
        let ctx = test_context();
        let other = mugraph_core::types::Keypair::random(&mut rand::rng());
        let mut note = keyset::issue_note(
            &mut rand::rng(),
            &other.secret_key,
            &Asset::default(),
            8,
        )
        .unwrap();
        note.delegate = ctx.keypair.public_key;

        let err = validate_notes(&[note], &ctx.keypair).unwrap_err();
        assert!(matches!(err, Error::InvalidSignature { .. }), "{err:?}");
    }

    #[test]
    fn validate_notes_rejects_a_note_for_another_delegate() {
        let ctx = test_context();
        let mut note = issue_notes(&ctx, 8).remove(0);
        note.delegate = mugraph_core::types::PublicKey::default();

        let err = validate_notes(&[note], &ctx.keypair).unwrap_err();
        assert!(format!("{err:?}").contains("delegate"), "{err:?}");
    }

    #[test]
    fn validate_notes_rejects_the_same_note_twice() {
        let ctx = test_context();
        let note = issue_notes(&ctx, 8).remove(0);

        let err =
            validate_notes(&[note.clone(), note], &ctx.keypair).unwrap_err();
        assert!(format!("{err:?}").contains("more than once"), "{err:?}");
    }

    #[test]
    fn validate_change_outputs_rejects_an_amount_that_is_not_a_denomination() {
        let ctx = test_context();
        let (mut outputs, _) = blinded_change(&ctx, 8);
        outputs[0].amount = 7;

        let err = validate_change_outputs(&outputs).unwrap_err();
        assert!(format!("{err:?}").contains("power of two"), "{err:?}");
    }

    #[tokio::test]
    async fn signers_must_include_the_node() {
        let (_, payment_vk) = generate_payment_keypair().unwrap();
        let wallet = mugraph_core::types::CardanoWallet::new(
            vec![],
            payment_vk.clone(),
            vec![],
            vec![],
            vault_address(),
            "preprod".to_string(),
        );
        let tx = build_tx([1u8; 32], &[to_user(1_000_000)], 170_000, &[], &[]);
        let parsed = ParsedWithdrawalTx::parse(&hex::encode(&tx.0)).unwrap();

        let err =
            validate_signers_with_parsed_tx(&parsed, &wallet).unwrap_err();
        assert!(format!("{err:?}").contains("required_signers"), "{err:?}");
    }

    #[tokio::test]
    async fn signers_accept_a_transaction_without_witnesses() {
        let (_, payment_vk) = generate_payment_keypair().unwrap();
        let wallet = mugraph_core::types::CardanoWallet::new(
            vec![],
            payment_vk.clone(),
            vec![],
            vec![],
            vault_address(),
            "preprod".to_string(),
        );
        let tx = build_tx(
            [1u8; 32],
            &[to_user(1_000_000)],
            170_000,
            &[node_hash(&payment_vk)],
            &[],
        );
        let parsed = ParsedWithdrawalTx::parse(&hex::encode(&tx.0)).unwrap();

        validate_signers_with_parsed_tx(&parsed, &wallet)
            .expect("the node adds its own witness later");
    }

    #[tokio::test]
    async fn signers_reject_an_invalid_witness() {
        let (_, payment_vk) = generate_payment_keypair().unwrap();
        let wallet = mugraph_core::types::CardanoWallet::new(
            vec![],
            payment_vk.clone(),
            vec![],
            vec![],
            vault_address(),
            "preprod".to_string(),
        );
        let signer = SigningKey::from_bytes(&[5u8; 32]);
        let (signed, _) = build_tx(
            [1u8; 32],
            &[to_user(1_000_000)],
            170_000,
            &[node_hash(&payment_vk)],
            &[&signer],
        );
        let (other, _) = build_tx(
            [1u8; 32],
            &[to_user(1_000_000)],
            170_001,
            &[node_hash(&payment_vk)],
            &[],
        );

        // Put the witness for one body on a different body.
        let signed = csl::Transaction::from_bytes(signed).unwrap();
        let other = csl::Transaction::from_bytes(other).unwrap();
        let tampered =
            csl::Transaction::new(&other.body(), &signed.witness_set(), None);
        let parsed =
            ParsedWithdrawalTx::parse(&hex::encode(tampered.to_bytes()))
                .unwrap();

        let err =
            validate_signers_with_parsed_tx(&parsed, &wallet).unwrap_err();
        assert!(matches!(err, Error::InvalidSignature { .. }), "{err:?}");
    }

    #[test]
    fn vault_outputs_must_carry_a_datum_for_this_node() {
        let (_, payment_vk) = generate_payment_keypair().unwrap();
        let hash = node_hash(&payment_vk);
        let wallet = mugraph_core::types::CardanoWallet::new(
            vec![],
            payment_vk,
            vec![],
            vec![],
            vault_address(),
            "preprod".to_string(),
        );

        let (good, _) = build_tx(
            [1u8; 32],
            &[to_user(900_000), to_vault(100_000, &hash)],
            170_000,
            std::slice::from_ref(&hash),
            &[],
        );
        let out = validate_network_and_vault_outputs(&good, &wallet).unwrap();
        assert_eq!(out, totals(&[("lovelace", 100_000)]));

        let no_datum = Out {
            datum_node_hash: None,
            ..to_vault(100_000, &hash)
        };
        let (bad, _) = build_tx(
            [1u8; 32],
            &[to_user(900_000), no_datum],
            170_000,
            std::slice::from_ref(&hash),
            &[],
        );
        let err =
            validate_network_and_vault_outputs(&bad, &wallet).unwrap_err();
        assert!(format!("{err:?}").contains("datum"), "{err:?}");

        let (other_node, _) = build_tx(
            [1u8; 32],
            &[to_user(900_000), to_vault(100_000, &[7u8; 28])],
            170_000,
            std::slice::from_ref(&hash),
            &[],
        );
        let err = validate_network_and_vault_outputs(&other_node, &wallet)
            .unwrap_err();
        assert!(format!("{err:?}").contains("node_pubkey_hash"), "{err:?}");
    }

    #[test]
    fn test_reject_output_wrong_network() {
        let tx = {
            let tx_hash =
                csl::TransactionHash::from_bytes(vec![0; 32]).unwrap();
            let mut inputs = csl::TransactionInputs::new();
            inputs.add(&csl::TransactionInput::new(&tx_hash, 0));
            let key_hash =
                csl::Ed25519KeyHash::from_bytes(vec![7u8; 28]).unwrap();
            let mainnet = csl::EnterpriseAddress::new(
                1,
                &csl::Credential::from_keyhash(&key_hash),
            )
            .to_address();
            let mut outputs = csl::TransactionOutputs::new();
            outputs.add(&csl::TransactionOutput::new(
                &mainnet,
                &csl::Value::new(&csl::Coin::from_str("1000000").unwrap()),
            ));
            let body = csl::TransactionBody::new_tx_body(
                &inputs,
                &outputs,
                &csl::Coin::from_str("170000").unwrap(),
            );
            csl::Transaction::new(
                &body,
                &csl::TransactionWitnessSet::new(),
                None,
            )
        };
        let wallet = mugraph_core::types::CardanoWallet::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vault_address(),
            "preprod".to_string(),
        );

        let err = validate_network_and_vault_outputs(&tx.to_bytes(), &wallet)
            .unwrap_err();
        assert!(format!("{:?}", err).contains("network_id 1"));
    }

    #[test]
    fn test_calculate_change_notes_signs_with_denomination_keys() {
        let ctx = test_context();
        let (outputs, pending) = blinded_change(&ctx, 100_000);
        let request = WithdrawRequest {
            notes: vec![],
            change_outputs: outputs,
            tx_cbor: String::new(),
            tx_hash: String::new(),
        };

        let signatures =
            calculate_change_notes(&request, &ctx.keypair).unwrap();
        assert_eq!(signatures.len(), pending.len());

        let keys = keyset::keyset(&ctx.keypair.secret_key, &Asset::default());
        for (pending, signature) in pending.into_iter().zip(&signatures) {
            let public_key =
                keyset::keyset_public_key(&keys, pending.note.amount).unwrap();
            pending
                .finish(signature, &public_key)
                .expect("change note verifies with its denomination key");
        }
    }

    // --- The full handler -------------------------------------------------

    #[tokio::test]
    async fn handle_withdraw_happy_path_burns_notes_and_completes() {
        let input_tx_hash = [0xabu8; 32];
        let s = scenario(input_tx_hash, StatusCode::OK, None).await;
        seed_deposit(
            &s.ctx,
            mugraph_core::types::UtxoRef::new(input_tx_hash, 0),
            [0u8; 32],
        );

        let response = handle_withdraw(&s.request, &s.ctx)
            .await
            .expect("withdraw accepted");
        assert!(matches!(response, Response::Withdraw { .. }));

        for note in &s.request.notes {
            assert!(note_is_burned(&s.ctx, note));
        }

        let read_tx = s.ctx.database.read().unwrap();
        let withdrawals = read_tx.open_table(WITHDRAWALS).unwrap();
        let key = withdrawal_key_from_hex(&s.request.tx_hash);
        assert_eq!(
            withdrawals.get(&key).unwrap().unwrap().value().status,
            mugraph_core::types::WithdrawalStatus::Completed
        );

        let deposits = read_tx.open_table(DEPOSITS).unwrap();
        assert!(
            deposits
                .get(mugraph_core::types::UtxoRef::new(input_tx_hash, 0))
                .unwrap()
                .unwrap()
                .value()
                .spent
        );
    }

    #[tokio::test]
    async fn handle_withdraw_spends_a_vault_utxo_without_a_deposit_record() {
        // The vault is one pool: a withdrawal can spend any vault UTxO,
        // for example the vault change of an earlier withdrawal.
        let s = scenario([0xaeu8; 32], StatusCode::OK, None).await;

        handle_withdraw(&s.request, &s.ctx)
            .await
            .expect("withdraw accepted");
    }

    #[tokio::test]
    async fn handle_withdraw_happy_path_returns_valid_change_notes() {
        let input_tx_hash = [0xacu8; 32];
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let hash = node_hash(&payment_vk);
        // 1.17 ADA in; 0.9 ADA to the user, 0.1 ADA back to the vault,
        // 0.17 ADA fee. Notes of 1.17 ADA burn, and 0.1 ADA comes back
        // as change notes.
        let tx = build_tx(
            input_tx_hash,
            &[to_user(900_000), to_vault(100_000, &hash)],
            170_000,
            std::slice::from_ref(&hash),
            &[],
        );
        let provider_url = spawn_withdraw_provider_mock(
            vault_address(),
            vault_datum_hex(&hash),
            1_170_000,
            StatusCode::OK,
            tx.1.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, &vault_address());

        let (change, pending) = blinded_change(&ctx, 100_000);
        let request = request(tx, issue_notes(&ctx, 1_170_000), change);

        let response = handle_withdraw(&request, &ctx)
            .await
            .expect("withdraw accepted");

        let Response::Withdraw { change_notes, .. } = response else {
            panic!("unexpected response");
        };
        let keys = keyset::keyset(&ctx.keypair.secret_key, &Asset::default());
        assert_eq!(change_notes.len(), pending.len());
        for (pending, signature) in pending.into_iter().zip(&change_notes) {
            let public_key =
                keyset::keyset_public_key(&keys, pending.note.amount).unwrap();
            pending.finish(signature, &public_key).unwrap();
        }
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_a_forged_note_without_mutating_state() {
        let mut s = scenario([0xb1u8; 32], StatusCode::OK, None).await;
        let other = mugraph_core::types::Keypair::random(&mut rand::rng());
        let mut forged = keyset::issue_note(
            &mut rand::rng(),
            &other.secret_key,
            &Asset::default(),
            s.request.notes[0].amount,
        )
        .unwrap();
        forged.delegate = s.ctx.keypair.public_key;
        s.request.notes[0] = forged;

        let err = handle_withdraw(&s.request, &s.ctx).await.unwrap_err();
        assert!(matches!(err, Error::InvalidSignature { .. }), "{err:?}");
        assert_preflight_rejection_leaves_state_untouched(&s.ctx, &s.request);
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_notes_worth_less_than_the_outflow() {
        let mut s = scenario([0xb2u8; 32], StatusCode::OK, None).await;
        // Keep only the smallest note: far less than 1.17 ADA.
        s.request.notes.truncate(1);

        let err = handle_withdraw(&s.request, &s.ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("does not match"), "{err:?}");
        assert_preflight_rejection_leaves_state_untouched(&s.ctx, &s.request);
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_a_tx_without_the_node_as_signer() {
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let hash = node_hash(&payment_vk);
        let tx =
            build_tx([0xb3u8; 32], &[to_user(1_000_000)], 170_000, &[], &[]);
        let provider_url = spawn_withdraw_provider_mock(
            vault_address(),
            vault_datum_hex(&hash),
            1_170_000,
            StatusCode::OK,
            tx.1.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, &vault_address());
        let request = request(tx, issue_notes(&ctx, 1_170_000), vec![]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("required_signers"), "{err:?}");
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request);
    }

    #[tokio::test]
    async fn handle_withdraw_hash_mismatch_does_not_mutate_state() {
        let mut s = scenario([0xb4u8; 32], StatusCode::OK, None).await;
        s.request.tx_hash = "ff".repeat(32);

        let err = handle_withdraw(&s.request, &s.ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("Transaction hash mismatch"));
        assert_preflight_rejection_leaves_state_untouched(&s.ctx, &s.request);
    }

    #[tokio::test]
    async fn handle_withdraw_balance_failure_does_not_mutate_state() {
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let hash = node_hash(&payment_vk);
        // The outputs and fee add up to more than the 1.17 ADA input.
        let tx = build_tx(
            [0xb5u8; 32],
            &[to_user(1_100_000)],
            170_000,
            std::slice::from_ref(&hash),
            &[],
        );
        let provider_url = spawn_withdraw_provider_mock(
            vault_address(),
            vault_datum_hex(&hash),
            1_170_000,
            StatusCode::OK,
            tx.1.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, &vault_address());
        let request = request(tx, issue_notes(&ctx, 1_170_000), vec![]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("Lovelace imbalance"), "{err:?}");
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request);
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_transactions_without_inputs() {
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let hash = node_hash(&payment_vk);
        let tx = {
            let inputs = csl::TransactionInputs::new();
            let mut outputs = csl::TransactionOutputs::new();
            outputs.add(&csl::TransactionOutput::new(
                &csl::Address::from_bech32(USER_ADDRESS).unwrap(),
                &csl::Value::new(&csl::Coin::from_str("1000000").unwrap()),
            ));
            let mut body = csl::TransactionBody::new_tx_body(
                &inputs,
                &outputs,
                &csl::Coin::from_str("170000").unwrap(),
            );
            let mut required = csl::Ed25519KeyHashes::new();
            required.add(&csl::Ed25519KeyHash::from_bytes(hash).unwrap());
            body.set_required_signers(&required);
            let tx = csl::Transaction::new(
                &body,
                &csl::TransactionWitnessSet::new(),
                None,
            );
            let cbor = tx.to_bytes();
            let hash = hex::encode(compute_tx_hash(&cbor).unwrap());
            (cbor, hash)
        };
        let ctx = test_context_with_provider_url(Some(
            "http://127.0.0.1:1".to_string(),
        ));
        insert_wallet(&ctx, payment_sk, payment_vk, &vault_address());
        let request = request(tx, issue_notes(&ctx, 1_170_000), vec![]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("No inputs found in transaction"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request);
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_inputs_not_from_script_address() {
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let hash = node_hash(&payment_vk);
        let tx = build_tx(
            [0xc4u8; 32],
            &[to_user(1_000_000)],
            170_000,
            std::slice::from_ref(&hash),
            &[],
        );
        let provider_url = spawn_withdraw_provider_mock(
            USER_ADDRESS.to_string(),
            vault_datum_hex(&hash),
            1_170_000,
            StatusCode::OK,
            tx.1.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, &vault_address());
        let request = request(tx, issue_notes(&ctx, 1_170_000), vec![]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("is not from script address"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request);
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_inputs_missing_inline_datum() {
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let hash = node_hash(&payment_vk);
        let tx = build_tx(
            [0xc5u8; 32],
            &[to_user(1_000_000)],
            170_000,
            std::slice::from_ref(&hash),
            &[],
        );
        let provider_url = spawn_withdraw_provider_mock_without_inline_datum(
            vault_address(),
            1_170_000,
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, &vault_address());
        let request = request(tx, issue_notes(&ctx, 1_170_000), vec![]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("missing inline datum"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request);
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_inputs_with_wrong_node_hash() {
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let hash = node_hash(&payment_vk);
        let tx = build_tx(
            [0xc6u8; 32],
            &[to_user(1_000_000)],
            170_000,
            std::slice::from_ref(&hash),
            &[],
        );
        let provider_url = spawn_withdraw_provider_mock(
            vault_address(),
            vault_datum_hex(&[7u8; 28]),
            1_170_000,
            StatusCode::OK,
            tx.1.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, &vault_address());
        let request = request(tx, issue_notes(&ctx, 1_170_000), vec![]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("node_pubkey_hash mismatch"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request);
    }

    #[tokio::test]
    async fn handle_withdraw_surfaces_provider_errors_without_mutating_state() {
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let hash = node_hash(&payment_vk);
        let tx = build_tx(
            [0xc7u8; 32],
            &[to_user(1_000_000)],
            170_000,
            std::slice::from_ref(&hash),
            &[],
        );
        let provider_url =
            spawn_withdraw_provider_mock_with_utxo_failure().await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, &vault_address());
        let request = request(tx, issue_notes(&ctx, 1_170_000), vec![]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("Failed to verify input 0"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request);
    }

    #[tokio::test]
    async fn handle_withdraw_submit_failure_marks_failed_without_unburning() {
        let s = scenario([0xcdu8; 32], StatusCode::INTERNAL_SERVER_ERROR, None)
            .await;

        let err = handle_withdraw(&s.request, &s.ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("Transaction submission failed"));

        for note in &s.request.notes {
            assert!(note_is_burned(&s.ctx, note));
        }
        let read_tx = s.ctx.database.read().unwrap();
        let withdrawals = read_tx.open_table(WITHDRAWALS).unwrap();
        let key = withdrawal_key_from_hex(&s.request.tx_hash);
        assert_eq!(
            withdrawals.get(&key).unwrap().unwrap().value().status,
            mugraph_core::types::WithdrawalStatus::Failed
        );
    }

    #[tokio::test]
    async fn handle_withdraw_mismatched_submit_hash_marks_failed() {
        let input_tx_hash = [0xc3u8; 32];
        let s = scenario(input_tx_hash, StatusCode::OK, Some("ee".repeat(32)))
            .await;
        let deposit_ref = mugraph_core::types::UtxoRef::new(input_tx_hash, 0);
        seed_deposit(&s.ctx, deposit_ref.clone(), [0u8; 32]);

        let err = handle_withdraw(&s.request, &s.ctx).await.unwrap_err();
        assert!(
            format!("{err:?}").contains("Provider returned mismatched tx hash")
        );

        for note in &s.request.notes {
            assert!(note_is_burned(&s.ctx, note));
        }
        let read_tx = s.ctx.database.read().unwrap();
        let withdrawals = read_tx.open_table(WITHDRAWALS).unwrap();
        let key = withdrawal_key_from_hex(&s.request.tx_hash);
        assert_eq!(
            withdrawals.get(&key).unwrap().unwrap().value().status,
            mugraph_core::types::WithdrawalStatus::Failed
        );
        let deposits = read_tx.open_table(DEPOSITS).unwrap();
        assert!(!deposits.get(&deposit_ref).unwrap().unwrap().value().spent);
        let _ = s.input_tx_hash;
    }

    // --- Withdrawal state -------------------------------------------------

    #[test]
    fn completion_state_failure_returns_error() {
        let result = finalize_withdraw_response(
            Err(Error::Internal {
                reason: "db write failed".to_string(),
            }),
            "deadbeef".to_string(),
            "ab".repeat(32),
            vec![],
        );

        assert!(result.is_err());
    }

    #[test]
    fn retry_after_failed_submission_does_not_reburn_notes() {
        let ctx = test_context();
        let request = WithdrawRequest {
            tx_hash: "ab".repeat(32),
            tx_cbor: "00".to_string(),
            notes: issue_notes(&ctx, 1),
            change_outputs: vec![],
        };

        atomic_burn_and_record_pending(&request, &ctx, &request.tx_hash)
            .expect("first burn succeeds");

        mark_withdrawal_failed(&ctx, &request.tx_hash).expect("mark as failed");

        check_idempotency(&request, &ctx)
            .expect("failed withdrawals are retryable");

        let second =
            atomic_burn_and_record_pending(&request, &ctx, &request.tx_hash);
        assert!(
            second.is_ok(),
            "retry should reuse failed pending state without burning notes again"
        );
    }

    #[test]
    fn completion_rejects_unknown_withdrawal_hash() {
        let ctx = test_context();
        let request = WithdrawRequest {
            tx_hash: "ab".repeat(32),
            tx_cbor: "00".to_string(),
            notes: issue_notes(&ctx, 2),
            change_outputs: vec![],
        };

        atomic_burn_and_record_pending(&request, &ctx, &request.tx_hash)
            .expect("first burn succeeds");

        let mismatched = "cd".repeat(32);
        let err =
            mark_withdrawal_completed(&ctx, &mismatched, &[]).unwrap_err();
        assert!(format!("{err:?}").contains("Pending withdrawal not found"));
    }

    #[test]
    fn completed_withdrawals_are_not_retryable() {
        let ctx = test_context();
        let request = WithdrawRequest {
            tx_hash: "ab".repeat(32),
            tx_cbor: "00".to_string(),
            notes: vec![],
            change_outputs: vec![],
        };
        seed_withdrawal_record(
            &ctx,
            &request.tx_hash,
            mugraph_core::types::WithdrawalRecord::completed(),
        );

        let err = check_idempotency(&request, &ctx).unwrap_err();
        assert!(format!("{err:?}").contains("Withdrawal already completed"));
    }

    #[test]
    fn completion_rejects_already_completed_withdrawal_without_mutating_deposits()
     {
        let ctx = test_context();
        let tx_hash = "cd".repeat(32);
        let utxo_ref = mugraph_core::types::UtxoRef::new([0xd0u8; 32], 0);
        seed_deposit(&ctx, utxo_ref.clone(), [0u8; 32]);
        seed_withdrawal_record(
            &ctx,
            &tx_hash,
            mugraph_core::types::WithdrawalRecord::completed(),
        );

        let err = mark_withdrawal_completed(
            &ctx,
            &tx_hash,
            std::slice::from_ref(&utxo_ref),
        )
        .unwrap_err();
        assert!(format!("{err:?}").contains("Withdrawal already completed"));

        let read_tx = ctx.database.read().unwrap();
        let deposits = read_tx.open_table(DEPOSITS).unwrap();
        assert!(!deposits.get(&utxo_ref).unwrap().unwrap().value().spent);
    }

    // --- Transaction builders ---------------------------------------------

    fn tx_hash_from_body(body: &csl::TransactionBody) -> csl::TransactionHash {
        type Blake2b256 = blake2::Blake2b<blake2::digest::consts::U32>;
        let tx_hash = Blake2b256::digest(body.to_bytes());
        let mut tx_hash_arr = [0u8; 32];
        tx_hash_arr.copy_from_slice(&tx_hash);
        csl::TransactionHash::from_bytes(tx_hash_arr.to_vec()).unwrap()
    }

    fn minimal_tx_with_values(
        output_lovelace: u64,
        fee: u64,
    ) -> csl::Transaction {
        let tx_hash = csl::TransactionHash::from_bytes(vec![0; 32]).unwrap();
        let input = csl::TransactionInput::new(&tx_hash, 0);
        let mut inputs = csl::TransactionInputs::new();
        inputs.add(&input);

        let addr = csl::Address::from_bech32(USER_ADDRESS).unwrap();
        let coin = csl::Coin::from_str(&output_lovelace.to_string()).unwrap();
        let value = csl::Value::new(&coin);
        let output = csl::TransactionOutput::new(&addr, &value);
        let mut outputs = csl::TransactionOutputs::new();
        outputs.add(&output);

        let fee = csl::Coin::from_str(&fee.to_string()).unwrap();
        let body = csl::TransactionBody::new_tx_body(&inputs, &outputs, &fee);
        let witness_set = csl::TransactionWitnessSet::new();
        csl::Transaction::new(&body, &witness_set, None)
    }

    fn tx_with_multiasset_output(
        lovelace: u64,
        assets: &[(&str, &str, u64)], // (policy_hex, asset_name_hex, qty)
    ) -> csl::Transaction {
        let tx_hash = csl::TransactionHash::from_bytes(vec![0; 32]).unwrap();
        let input = csl::TransactionInput::new(&tx_hash, 0);
        let mut inputs = csl::TransactionInputs::new();
        inputs.add(&input);

        let addr = csl::Address::from_bech32(USER_ADDRESS).unwrap();
        let coin = csl::Coin::from_str(&lovelace.to_string()).unwrap();
        let mut value = csl::Value::new(&coin);

        if !assets.is_empty() {
            let mut ma = csl::MultiAsset::new();
            for (policy_hex, asset_hex, qty) in assets {
                let policy = csl::ScriptHash::from_hex(policy_hex).unwrap();
                let mut assets_map = ma.get(&policy).unwrap_or_default();
                let name_bytes = hex::decode(asset_hex).unwrap();
                let name = csl::AssetName::new(name_bytes).unwrap();
                assets_map.insert(
                    &name,
                    &csl::BigNum::from_str(&qty.to_string()).unwrap(),
                );
                ma.insert(&policy, &assets_map);
            }
            value.set_multiasset(&ma);
        }

        let output = csl::TransactionOutput::new(&addr, &value);
        let mut outputs = csl::TransactionOutputs::new();
        outputs.add(&output);

        let fee = csl::Coin::from_str("0").unwrap(); // fee handled separately in tests
        let body = csl::TransactionBody::new_tx_body(&inputs, &outputs, &fee);
        let witness_set = csl::TransactionWitnessSet::new();
        csl::Transaction::new(&body, &witness_set, None)
    }
}
