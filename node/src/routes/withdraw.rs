use std::collections::{HashMap, HashSet};

use blake2::Digest;
use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{
        BlindSignature,
        Keypair,
        Response,
        Signature,
        WithdrawRequest,
        WithdrawalRecord,
        WithdrawalStatus,
    },
};
use redb::ReadableTable;
use whisky_csl::csl;

use crate::{
    database::{CARDANO_WALLET, NOTES, WITHDRAWALS},
    deposit_datum::{DepositDatumContext, parse_deposit_datum},
    network::CardanoNetwork,
    provider::Provider,
    routes::Context,
    tx_signer::{attach_witness_to_transaction, compute_tx_hash},
};

struct ParsedWithdrawalTx {
    tx_cbor: Vec<u8>,
    tx: csl::Transaction,
    tx_hash: [u8; 32],
    tx_hash_hex: String,
}

impl ParsedWithdrawalTx {
    fn parse(tx_cbor_hex: &str) -> Result<Self, Error> {
        let tx_cbor = hex::decode(tx_cbor_hex).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid tx_cbor hex: {}", e),
        })?;
        let tx = csl::Transaction::from_bytes(tx_cbor.clone()).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid transaction CBOR: {}", e),
        })?;
        let tx_hash = compute_tx_hash(&tx_cbor).map_err(|e| Error::InvalidInput {
            reason: format!("Failed to compute tx hash: {}", e),
        })?;

        Ok(Self {
            tx_cbor,
            tx,
            tx_hash_hex: hex::encode(tx_hash),
            tx_hash,
        })
    }
}

/// Handle withdrawal request
///
/// 1. Parse and validate the withdrawal request
/// 2. Verify transaction CBOR and recompute hash
/// 3. Ensure all inputs reference script UTxOs
/// 4. Validate user signatures (transaction witnesses via whisky-csl)
/// 5. Check outputs match burned notes minus fees
/// 6. Burn notes
/// 7. Attach node witness and re-serialize
/// 8. Submit transaction to provider
/// 9. Return signed CBOR + hash + change notes
pub async fn handle_withdraw(request: &WithdrawRequest, ctx: &Context) -> Result<Response, Error> {
    tracing::info!(
        "Processing withdrawal request for tx_hash: {}",
        &request.tx_hash[..std::cmp::min(16, request.tx_hash.len())]
    );

    // 1. Preflight validation
    let provider = create_provider(ctx)?;
    let parsed_tx = ParsedWithdrawalTx::parse(&request.tx_cbor)?;

    // 2. Check idempotency via WITHDRAWALS table
    check_idempotency(request, ctx)?;

    // 3. Validate transaction size and fee
    if parsed_tx.tx_cbor.len() > ctx.config.max_tx_size() {
        return Err(Error::InvalidInput {
            reason: format!(
                "Transaction size {} bytes exceeds maximum {} bytes",
                parsed_tx.tx_cbor.len(),
                ctx.config.max_tx_size()
            ),
        });
    }

    // Validate fee with tolerance
    let _fee = validate_parsed_fee(
        &parsed_tx,
        ctx.config.max_withdrawal_fee(),
        ctx.config.fee_tolerance_pct(),
    )?;

    // 4. Load wallet needed for validations and signing
    let wallet = load_wallet(ctx)?;

    // 5. Verify provided hash matches recomputed hash
    let computed_hash = parsed_tx.tx_hash_hex.clone();
    if computed_hash != request.tx_hash {
        return Err(Error::InvalidInput {
            reason: format!(
                "Transaction hash mismatch: computed {}, provided {}",
                computed_hash, request.tx_hash
            ),
        });
    }

    // 6. Ensure all inputs reference script UTxOs and validate deposit state
    let (input_totals, required_user_hashes, consumed_deposits) =
        validate_script_inputs_with_parsed_tx(&parsed_tx, &wallet, ctx, &provider).await?;

    // 6b. Enforce intent and network binding via auxiliary metadata
    validate_withdraw_intent_metadata_with_parsed_tx(&parsed_tx, &wallet.network)?;

    // 7. Validate user witnesses (basic count check)
    validate_user_witnesses_with_parsed_tx(
        &parsed_tx,
        &request.notes,
        &required_user_hashes,
        &wallet,
    )
    .await?;

    // 8. Validate transaction value balance
    validate_transaction_balance_with_parsed_tx(
        &parsed_tx,
        &input_totals,
        ctx.config.max_withdrawal_fee(),
        ctx.config.fee_tolerance_pct(),
    )?;

    // 9. Enforce network consistency and reject change back to the script
    validate_network_and_change_outputs_with_parsed_tx(&parsed_tx, &wallet)?;

    // 9. Create signed transaction (without burning notes yet)
    // This prepares the transaction for submission but doesn't modify state

    // Node signature is attached to the transaction witness set
    // The validator checks that the transaction is properly signed (off-chain verification)
    // No redeemer is needed - all validation happens through witnesses

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
    let change_notes = calculate_change_notes(request, &parsed_tx.tx_cbor, &wallet, &ctx.keypair)?;

    // 9. Update state atomically BEFORE submitting to provider
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

    // 10. Submit transaction to provider
    let submit_response = match submit_transaction(&signed_cbor_hex, &provider).await {
        Ok(response) => response,
        Err(e) => {
            // Submission can fail ambiguously (e.g. timeout after relay acceptance).
            // Keep burned/pending state for deterministic reconciliation instead of unburning.
            tracing::error!(
                "Transaction submission failed after notes were burned: {}. Marking withdrawal failed for recovery.",
                e
            );
            if let Err(mark_err) = mark_withdrawal_failed(ctx, &pending_tx_hash) {
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

    // 11. Mark withdrawal as completed
    let mark_result = mark_withdrawal_completed(ctx, &pending_tx_hash, &consumed_deposits);

    finalize_withdraw_response(
        mark_result,
        signed_cbor_hex,
        pending_tx_hash,
        change_notes,
    )
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

/// Create Cardano provider from configuration
fn create_provider(ctx: &Context) -> Result<Provider, Error> {
    Provider::new(
        &ctx.config.provider_type(),
        ctx.config.provider_api_key(),
        ctx.config.network(),
        ctx.config.provider_url(),
    )
    .map_err(|e| Error::Internal {
        reason: e.to_string(),
    })
}

/// Check if withdrawal has already been processed (idempotency)
fn check_idempotency(request: &WithdrawRequest, ctx: &Context) -> Result<(), Error> {
    let read_tx = ctx.database.read()?;
    let table = read_tx.open_table(WITHDRAWALS)?;

    // Use network byte from config
    let network_byte = ctx.config.network_byte();
    let key = crate::tx_ids::parse_withdrawal_key(&request.tx_hash, network_byte)?;

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

/// Atomically burn notes and record withdrawal as pending
///
/// This is the first step in the withdrawal process.
/// Notes are burned and withdrawal is recorded before submission.
fn atomic_burn_and_record_pending(
    request: &WithdrawRequest,
    ctx: &Context,
    tx_hash: &str,
) -> Result<(), Error> {
    let network_byte = ctx.config.network_byte();
    let key = crate::tx_ids::parse_withdrawal_key(tx_hash, network_byte)?;

    // Retry path after ambiguous submission: notes are already burned for this tx.
    {
        let read_tx = ctx.database.read()?;
        let withdrawals = read_tx.open_table(WITHDRAWALS)?;
        if let Some(existing) = withdrawals.get(&key)?
            && existing.value().status == WithdrawalStatus::Failed {
                let write_tx = ctx.database.write()?;
                {
                    let mut withdrawals_table = write_tx.open_table(WITHDRAWALS)?;
                    withdrawals_table.insert(&key, &WithdrawalRecord::pending())?;
                }
                write_tx.commit()?;
                tracing::info!(
                    "Reused failed withdrawal state for retry without reburning notes: {}",
                    &tx_hash[..std::cmp::min(16, tx_hash.len())]
                );
                return Ok(());
            }
    }

    let write_tx = ctx.database.write()?;

    {
        // 1. Burn notes
        let mut notes_table = write_tx.open_table(NOTES)?;

        for note in &request.notes {
            let sig_bytes: &[u8; 32] = note.signature.0.as_ref();
            let signature = Signature::from(*sig_bytes);

            // Check if note is already spent
            if notes_table.get(signature)?.is_some() {
                return Err(Error::AlreadySpent { signature });
            }

            // Mark note as spent
            notes_table.insert(signature, true)?;
        }

        // 2. Record withdrawal as pending
        let mut withdrawals_table = write_tx.open_table(WITHDRAWALS)?;
        withdrawals_table.insert(&key, &WithdrawalRecord::pending())?;
    }

    write_tx.commit()?;

    tracing::info!(
        "Burned {} notes and recorded pending withdrawal {}",
        request.notes.len(),
        &tx_hash[..std::cmp::min(16, tx_hash.len())]
    );

    Ok(())
}

/// Mark withdrawal as failed for recovery
fn mark_withdrawal_failed(ctx: &Context, tx_hash: &str) -> Result<(), Error> {
    let write_tx = ctx.database.write()?;

    {
        let mut withdrawals_table = write_tx.open_table(WITHDRAWALS)?;

        let network_byte = ctx.config.network_byte();
        let key = crate::tx_ids::parse_withdrawal_key(tx_hash, network_byte)?;
        let record = WithdrawalRecord::failed();
        withdrawals_table.insert(key, &record)?;
    }

    write_tx.commit()?;
    Ok(())
}

/// Mark withdrawal as completed after successful submission and lock consumed deposits.
fn mark_withdrawal_completed(
    ctx: &Context,
    tx_hash: &str,
    consumed_deposits: &[mugraph_core::types::UtxoRef],
) -> Result<(), Error> {
    use crate::database::DEPOSITS;

    let write_tx = ctx.database.write()?;

    {
        let mut withdrawals_table = write_tx.open_table(WITHDRAWALS)?;

        let network_byte = ctx.config.network_byte();
        let key = crate::tx_ids::parse_withdrawal_key(tx_hash, network_byte)?;

        let existing = withdrawals_table.get(&key)?.map(|v| v.value());
        let Some(existing) = existing else {
            return Err(Error::InvalidInput {
                reason: "Pending withdrawal not found for completion".to_string(),
            });
        };

        if existing.status == WithdrawalStatus::Completed {
            return Err(Error::InvalidInput {
                reason: "Withdrawal already completed".to_string(),
            });
        }

        // Update record to mark as completed
        let record = WithdrawalRecord::completed();
        withdrawals_table.insert(key, &record)?;

        let mut deposits_table = write_tx.open_table(DEPOSITS)?;
        for utxo_ref in consumed_deposits {
            let existing_record = deposits_table.get(utxo_ref)?.map(|v| v.value());
            if let Some(mut record) = existing_record {
                record.spent = true;
                deposits_table.insert(utxo_ref, &record)?;
            }
        }
    }

    write_tx.commit()?;

    tracing::info!("Marked withdrawal {} as completed", tx_hash);

    Ok(())
}

/// Load Cardano wallet for signing
fn load_wallet(ctx: &Context) -> Result<mugraph_core::types::CardanoWallet, Error> {
    let read_tx = ctx.database.read()?;
    let table = read_tx.open_table(CARDANO_WALLET)?;

    match table.get("wallet")? {
        Some(wallet) => Ok(wallet.value()),
        None => Err(Error::Internal {
            reason: "Cardano wallet not initialized".to_string(),
        }),
    }
}

/// Submit transaction to Cardano provider
async fn submit_transaction(
    tx_cbor: &str,
    provider: &Provider,
) -> Result<crate::provider::SubmitResponse, Error> {
    let tx_bytes = hex::decode(tx_cbor).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid signed CBOR hex: {}", e),
    })?;

    provider
        .submit_tx(&tx_bytes)
        .await
        .map_err(|e| Error::NetworkError {
            reason: format!("Failed to submit transaction: {}", e),
        })
}

/// Validate transaction fee is within acceptable bounds
///
/// # Arguments
/// * `tx_cbor` - The transaction CBOR bytes
/// * `max_fee_lovelace` - Maximum acceptable fee in lovelace
/// * `tolerance_pct` - Fee tolerance percentage (0-100) for acceptable variance
///
/// # Returns
/// The fee amount in lovelace if valid
fn validate_parsed_fee(
    parsed_tx: &ParsedWithdrawalTx,
    max_fee_lovelace: u64,
    tolerance_pct: u8,
) -> Result<u64, Error> {
    let fee = extract_transaction_fee_from_tx(&parsed_tx.tx)?;

    validate_fee_amount(fee, max_fee_lovelace, tolerance_pct)
}

fn validate_fee_amount(
    fee: u64,
    max_fee_lovelace: u64,
    tolerance_pct: u8,
) -> Result<u64, Error> {

    // Calculate acceptable fee range with tolerance.
    // If tolerance is 5%, fee can be up to 105% of max_fee.
    let tolerance_factor = 100 + tolerance_pct as u64;
    let max_acceptable_fee = max_fee_lovelace.saturating_mul(tolerance_factor) / 100;

    if fee > max_acceptable_fee {
        return Err(Error::InvalidInput {
            reason: format!(
                "Fee {} lovelace exceeds acceptable maximum {} lovelace (base max {}, tolerance {}%)",
                fee, max_acceptable_fee, max_fee_lovelace, tolerance_pct
            ),
        });
    }

    tracing::info!(
        "Transaction fee: {} lovelace (max: {}, tolerance: {}%, acceptable max: {})",
        fee,
        max_fee_lovelace,
        tolerance_pct,
        max_acceptable_fee
    );
    Ok(fee)
}

/// Extract fee from transaction body CBOR
fn extract_transaction_fee_from_tx(tx: &csl::Transaction) -> Result<u64, Error> {
    let fee_str = tx.body().fee().to_str();
    fee_str.parse::<u64>().map_err(|e| Error::InvalidInput {
        reason: format!("Failed to parse fee: {}", e),
    })
}

/// Extract transaction inputs from CBOR
///
/// Transaction body structure (simplified):
/// - inputs: []TransactionInput (array of {transaction_id, index})
/// - outputs: []TransactionOutput
/// - fee: Coin
/// - ... other fields
fn extract_transaction_inputs_from_tx(tx: &csl::Transaction) -> Result<Vec<(Vec<u8>, u32)>, Error> {
    let inputs: Vec<(Vec<u8>, u32)> = (&tx.body().inputs())
        .into_iter()
        .map(|input| {
            let tx_id = input.transaction_id().to_bytes();
            let idx = input.index();
            (tx_id, idx)
        })
        .collect();

    if inputs.is_empty() {
        return Err(Error::InvalidInput {
            reason: "No inputs found in transaction".to_string(),
        });
    }

    Ok(inputs)
}

fn checked_output_index(index: u32, input_pos: usize) -> Result<u16, Error> {
    u16::try_from(index).map_err(|_| Error::InvalidInput {
        reason: format!(
            "Input {} has output index {} which exceeds u16::MAX",
            input_pos, index
        ),
    })
}

/// Validate user witnesses
///
/// Uses whisky-csl (cardano-serialization-lib bindings) to parse the transaction,
/// compute the tx body hash (BLAKE2b-256) and verify each vkey / bootstrap witness
/// signature against that hash.
///
/// # Arguments
/// * `tx_cbor` - The transaction CBOR bytes
/// * `notes` - The notes being withdrawn (for verification)
/// * `wallet` - The Cardano wallet for address information
async fn validate_user_witnesses_with_parsed_tx(
    parsed_tx: &ParsedWithdrawalTx,
    notes: &[mugraph_core::types::BlindSignature],
    expected_user_hashes: &HashSet<String>,
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<(), Error> {
    validate_user_witnesses_from_tx(
        &parsed_tx.tx,
        &parsed_tx.tx_hash,
        notes,
        expected_user_hashes,
        wallet,
    )
    .await
}

#[cfg(test)]
async fn validate_user_witnesses(
    tx_cbor: &[u8],
    notes: &[mugraph_core::types::BlindSignature],
    expected_user_hashes: &HashSet<String>,
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<(), Error> {
    let parsed_tx = ParsedWithdrawalTx::parse(&hex::encode(tx_cbor))?;
    validate_user_witnesses_with_parsed_tx(&parsed_tx, notes, expected_user_hashes, wallet).await
}

async fn validate_user_witnesses_from_tx(
    tx: &csl::Transaction,
    tx_hash: &[u8; 32],
    notes: &[mugraph_core::types::BlindSignature],
    expected_user_hashes: &HashSet<String>,
    _wallet: &mugraph_core::types::CardanoWallet,
) -> Result<(), Error> {
    let body_hash_bytes = tx_hash.to_vec();

    let (witness_key_hashes, verified_witnesses) =
        verify_witness_set(tx, &body_hash_bytes)?;
    let required_signer_hashes = collect_required_signer_hashes(tx)?;

    ensure_required_signers_have_witnesses(&required_signer_hashes, &witness_key_hashes)?;
    ensure_expected_owner_hashes_are_bound(
        expected_user_hashes,
        &required_signer_hashes,
        &witness_key_hashes,
    )?;

    tracing::info!(
        "Validated {} witness signatures for {} notes",
        verified_witnesses,
        notes.len()
    );

    Ok(())
}

fn verify_witness_set(
    tx: &csl::Transaction,
    body_hash_bytes: &[u8],
) -> Result<(HashSet<String>, usize), Error> {
    let witness_set = tx.witness_set();
    let mut verified_witnesses = 0usize;
    let mut witness_key_hashes = HashSet::new();

    if let Some(vkeys) = witness_set.vkeys() {
        for (idx, witness) in (&vkeys).into_iter().enumerate() {
            let pk: csl::PublicKey = witness.vkey().public_key();
            let sig = witness.signature();
            if !pk.verify(body_hash_bytes, &sig) {
                return Err(Error::InvalidSignature {
                    reason: format!("VKey witness {} signature invalid", idx),
                    signature: mugraph_core::types::Signature::default(),
                });
            }
            witness_key_hashes.insert(pk.hash().to_hex());
            verified_witnesses += 1;
        }
    }

    if let Some(bootstraps) = witness_set.bootstraps() {
        for (idx, witness) in (&bootstraps).into_iter().enumerate() {
            let pk: csl::PublicKey = witness.vkey().public_key();
            let sig = witness.signature();
            if !pk.verify(body_hash_bytes, &sig) {
                return Err(Error::InvalidSignature {
                    reason: format!("Bootstrap witness {} signature invalid", idx),
                    signature: mugraph_core::types::Signature::default(),
                });
            }
            witness_key_hashes.insert(pk.hash().to_hex());
            verified_witnesses += 1;
        }
    }

    if verified_witnesses == 0 {
        return Err(Error::InvalidSignature {
            reason: "No valid witnesses found in transaction".to_string(),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    Ok((witness_key_hashes, verified_witnesses))
}

fn collect_required_signer_hashes(tx: &csl::Transaction) -> Result<Vec<String>, Error> {
    let required = tx.body().required_signers().ok_or_else(|| Error::InvalidSignature {
        reason: "Transaction missing required_signers; cannot bind witnesses to note owners"
            .to_string(),
        signature: mugraph_core::types::Signature::default(),
    })?;

    Ok(required.into_iter().map(|signer| signer.to_hex()).collect())
}

fn ensure_required_signers_have_witnesses(
    required_signer_hashes: &[String],
    witness_key_hashes: &HashSet<String>,
) -> Result<(), Error> {
    let missing: Vec<String> = required_signer_hashes
        .iter()
        .filter(|signer_hash| !witness_key_hashes.contains(*signer_hash))
        .cloned()
        .collect();

    if missing.is_empty() {
        return Ok(());
    }

    Err(Error::InvalidSignature {
        reason: format!("Missing witnesses for required_signers: {:?}", missing),
        signature: mugraph_core::types::Signature::default(),
    })
}

fn ensure_expected_owner_hashes_are_bound(
    expected_user_hashes: &HashSet<String>,
    required_signer_hashes: &[String],
    witness_key_hashes: &HashSet<String>,
) -> Result<(), Error> {
    if expected_user_hashes.is_empty() {
        return Err(Error::InvalidSignature {
            reason: "No expected user hashes derived from inputs".to_string(),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    for expected in expected_user_hashes {
        if !required_signer_hashes.iter().any(|signer_hash| signer_hash == expected) {
            return Err(Error::InvalidSignature {
                reason: format!(
                    "Required signer set does not include input owner hash {}",
                    expected
                ),
                signature: mugraph_core::types::Signature::default(),
            });
        }

        if !witness_key_hashes.contains(expected) {
            return Err(Error::InvalidSignature {
                reason: format!("Missing witness for input owner hash {}", expected),
                signature: mugraph_core::types::Signature::default(),
            });
        }
    }

    Ok(())
}

/// Calculate change notes from transaction outputs
///
/// # Implementation Note
/// This parses the transaction to identify change outputs and calculates
/// the total change amount. A full implementation would create blind signatures
/// for the change notes.
///
/// # Arguments
/// * `request` - The withdrawal request
/// * `tx_cbor` - The transaction CBOR bytes
/// * `wallet` - The Cardano wallet for address comparison
///
/// # Returns
/// Empty vector for now - change notes will be implemented when the full
/// blinding infrastructure is in place
fn calculate_change_notes(
    _request: &WithdrawRequest,
    _tx_cbor: &[u8],
    _wallet: &mugraph_core::types::CardanoWallet,
    _keypair: &Keypair,
) -> Result<Vec<BlindSignature>, Error> {
    // Do not fabricate synthetic change notes from address/amount material.
    // Change note issuance must be driven by protocol-bound blinded commitments.
    Ok(Vec::new())
}

/// Validate script inputs and check deposit state
///
/// This function:
/// 1. Extracts inputs from the transaction
/// 2. Queries the blockchain to verify each input is at the script address
/// 3. Checks the DEPOSITS table to ensure deposits are valid (not already spent, not expired)
/// 4. Returns the total value of inputs being spent
///
/// # Arguments
/// * `tx_cbor` - The transaction CBOR bytes
/// * `script_address` - The expected script address
/// * `ctx` - The request context (for database access)
/// * `provider` - The blockchain provider
///
/// # Returns
/// A vector of (tx_hash, amount) tuples for each valid input
async fn validate_script_inputs_with_parsed_tx(
    parsed_tx: &ParsedWithdrawalTx,
    wallet: &mugraph_core::types::CardanoWallet,
    ctx: &Context,
    provider: &Provider,
) -> Result<(
    HashMap<String, u128>,
    HashSet<String>,
    Vec<mugraph_core::types::UtxoRef>,
), Error> {
    let inputs = extract_transaction_inputs_from_tx(&parsed_tx.tx)?;
    validate_script_inputs_with_extracted_inputs(inputs, wallet, ctx, provider).await
}

async fn validate_script_inputs_with_extracted_inputs(
    inputs: Vec<(Vec<u8>, u32)>,
    wallet: &mugraph_core::types::CardanoWallet,
    ctx: &Context,
    provider: &Provider,
) -> Result<(
    HashMap<String, u128>,
    HashSet<String>,
    Vec<mugraph_core::types::UtxoRef>,
), Error> {
    use mugraph_core::types::UtxoRef;

    use crate::database::DEPOSITS;

    if inputs.is_empty() {
        return Err(Error::InvalidInput {
            reason: "Transaction has no inputs".to_string(),
        });
    }

    let mut totals: HashMap<String, u128> = HashMap::new();
    let mut required_user_hashes: HashSet<String> = HashSet::new();
    let mut consumed_deposits: Vec<UtxoRef> = Vec::new();
    let read_tx = ctx.database.read()?;
    let deposits_table = read_tx.open_table(DEPOSITS)?;

    // Pre-compute node pubkey hash (blake2b-224) to compare with datum
    let node_pk = csl::PublicKey::from_bytes(&wallet.payment_vk).map_err(|e| Error::InvalidKey {
        reason: format!("Invalid node payment_vk: {}", e),
    })?;
    let node_pk_hash: [u8; 28] = node_pk
        .hash()
        .to_bytes()
        .try_into()
        .expect("Cardano key hashes are always 28 bytes");

    for (i, (tx_hash_bytes, index)) in inputs.iter().enumerate() {
        let tx_hash = hex::encode(tx_hash_bytes);
        let output_index = checked_output_index(*index, i)?;

        tracing::debug!("Validating input {}: {}:{}", i, &tx_hash[..16], index);

        // Query blockchain to verify input is at script address
        match provider.get_utxo(&tx_hash, output_index).await {
            Ok(Some(utxo_info)) => {
                if utxo_info.address != wallet.script_address {
                    return Err(Error::InvalidInput {
                        reason: format!(
                            "Input {} ({}:{}) is not from script address. Expected {}, got {}",
                            i,
                            &tx_hash[..16],
                            index,
                            wallet.script_address,
                            utxo_info.address
                        ),
                    });
                }

                // Datum must be present to bind witness to owner
                let datum_hex = utxo_info
                    .datum
                    .as_ref()
                    .ok_or_else(|| Error::InvalidInput {
                        reason: format!(
                            "Input {} ({}:{}) missing inline datum; required for witness binding",
                            i,
                            &tx_hash[..16],
                            index
                        ),
                    })?;

                let datum = parse_deposit_datum(
                    datum_hex,
                    DepositDatumContext::WithdrawalInput { input_index: i },
                )?;

                required_user_hashes.insert(hex::encode(datum.user_pubkey_hash));

                if datum.node_pubkey_hash != node_pk_hash {
                    return Err(Error::InvalidInput {
                        reason: format!(
                            "Input {} node_pubkey_hash mismatch; expected our node, got {}",
                            i,
                            hex::encode(datum.node_pubkey_hash)
                        ),
                    });
                }

                // Calculate total value of this UTxO
                for asset in &utxo_info.amount {
                    let qty: u128 =
                        asset
                            .quantity
                            .parse::<u128>()
                            .map_err(|e| Error::InvalidInput {
                                reason: format!("Invalid asset quantity: {}", e),
                            })?;
                    let entry = totals.entry(asset.unit.clone()).or_insert(0);
                    *entry = entry.saturating_add(qty);
                }

                // Check deposit state in our database
                let tx_hash_array: [u8; 32] =
                    tx_hash_bytes
                        .as_slice()
                        .try_into()
                        .map_err(|_| Error::InvalidInput {
                            reason: format!("Invalid tx_hash length for input {}", i),
                        })?;
                let utxo_ref = UtxoRef::new(tx_hash_array, output_index);
                consumed_deposits.push(utxo_ref.clone());

                match deposits_table.get(&utxo_ref)? {
                    Some(deposit) => {
                        let deposit_record = deposit.value();
                        if deposit_record.spent {
                            return Err(Error::InvalidInput {
                                reason: format!(
                                    "Input {} ({}:{}) deposit already spent",
                                    i,
                                    &tx_hash[..16],
                                    index
                                ),
                            });
                        }

                        // Ensure intent_hash matches what we recorded (if present)
                        if deposit_record.intent_hash != [0u8; 32]
                            && datum.intent_hash != deposit_record.intent_hash
                        {
                            return Err(Error::InvalidInput {
                                reason: format!(
                                    "Intent hash mismatch for input {}: datum {}, expected {}",
                                    i,
                                    hex::encode(datum.intent_hash),
                                    hex::encode(deposit_record.intent_hash)
                                ),
                            });
                        }

                        // Check if expired
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        if now > deposit_record.expires_at {
                            return Err(Error::InvalidInput {
                                reason: format!(
                                    "Input {} ({}:{}) deposit expired at {}",
                                    i,
                                    &tx_hash[..16],
                                    index,
                                    deposit_record.expires_at
                                ),
                            });
                        }

                        tracing::info!(
                            "Input {}: deposit valid (block {}, expires {})",
                            i,
                            deposit_record.block_height,
                            deposit_record.expires_at
                        );
                    }
                    None => {
                        // Deposit not in our database - might be a fresh deposit not yet recorded
                        // or an invalid input. For security, we should reject unknown deposits.
                        tracing::warn!(
                            "Input {} ({}:{}) not found in DEPOSITS table",
                            i,
                            &tx_hash[..16],
                            index
                        );
                        return Err(Error::InvalidInput {
                            reason: format!(
                                "Input {} ({}:{}) deposit not found. Deposits must be recorded before withdrawal.",
                                i,
                                &tx_hash[..16],
                                index
                            ),
                        });
                    }
                }
            }
            Ok(None) => {
                return Err(Error::InvalidInput {
                    reason: format!(
                        "Input {} ({}:{}) not found on chain",
                        i,
                        &tx_hash[..16],
                        index
                    ),
                });
            }
            Err(e) => {
                return Err(Error::NetworkError {
                    reason: format!("Failed to verify input {}: {}", i, e),
                });
            }
        }
    }

    tracing::info!(
        "Aggregated input totals: {:?}",
        totals
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<_>>()
            .join(", ")
    );

    Ok((totals, required_user_hashes, consumed_deposits))
}

fn max_acceptable_fee(max_fee_lovelace: u64, tolerance_pct: u8) -> u64 {
    let tolerance_factor = 100 + tolerance_pct as u64;
    max_fee_lovelace.saturating_mul(tolerance_factor) / 100
}

fn validate_transaction_balance_with_parsed_tx(
    parsed_tx: &ParsedWithdrawalTx,
    input_totals: &HashMap<String, u128>,
    max_fee: u64,
    fee_tolerance_pct: u8,
) -> Result<(), Error> {
    let effective_max_fee = max_acceptable_fee(max_fee, fee_tolerance_pct);
    validate_transaction_balance_from_tx(&parsed_tx.tx, input_totals, effective_max_fee)
}

#[cfg(test)]
fn validate_transaction_balance_with_tolerance(
    tx_cbor: &[u8],
    input_totals: &HashMap<String, u128>,
    max_fee: u64,
    fee_tolerance_pct: u8,
) -> Result<(), Error> {
    let effective_max_fee = max_acceptable_fee(max_fee, fee_tolerance_pct);
    validate_transaction_balance(tx_cbor, input_totals, effective_max_fee)
}

/// Validate transaction balance
///
/// Verifies that: inputs - fee = outputs (within tolerance)
/// This ensures the transaction conserves value properly.
///
/// # Arguments
/// * `tx_cbor` - The transaction CBOR bytes
/// * `total_input` - Total value of all inputs (from deposits)
/// * `max_fee` - Maximum acceptable fee
///
/// # Returns
/// Ok if the balance is valid, Err otherwise
#[cfg(test)]
fn validate_transaction_balance(
    tx_cbor: &[u8],
    input_totals: &HashMap<String, u128>,
    max_fee: u64,
) -> Result<(), Error> {
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec()).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction CBOR: {}", e),
    })?;

    validate_transaction_balance_from_tx(&tx, input_totals, max_fee)
}

fn validate_transaction_balance_from_tx(
    tx: &csl::Transaction,
    input_totals: &HashMap<String, u128>,
    max_fee: u64,
) -> Result<(), Error> {
    let fee_u128: u128 = tx
        .body()
        .fee()
        .to_str()
        .parse()
        .map_err(|e| Error::InvalidInput {
            reason: format!("Invalid fee: {}", e),
        })?;

    if fee_u128 > max_fee as u128 {
        return Err(Error::InvalidInput {
            reason: format!("Fee {} exceeds maximum {}", fee_u128, max_fee),
        });
    }

    // Aggregate outputs by unit
    let mut output_totals: HashMap<String, u128> = HashMap::new();
    for output in &tx.body().outputs() {
        let coin = output.amount().coin();
        let entry = output_totals.entry("lovelace".to_string()).or_insert(0);
        *entry =
            entry.saturating_add(
                coin.to_str()
                    .parse::<u128>()
                    .map_err(|e| Error::InvalidInput {
                        reason: format!("Invalid lovelace amount: {}", e),
                    })?,
            );

        if let Some(ma) = output.amount().multiasset() {
            let policies = ma.keys();
            for idx in 0..policies.len() {
                let policy = policies.get(idx);
                if let Some(assets) = ma.get(&policy) {
                    let names = assets.keys();
                    for j in 0..names.len() {
                        let asset_name = names.get(j);
                        let qty = assets.get(&asset_name).unwrap();
                        let unit = format!("{}{}", policy.to_hex(), asset_name.to_hex());
                        let e = output_totals.entry(unit).or_insert(0);
                        *e = e.saturating_add(qty.to_str().parse::<u128>().map_err(|e| {
                            Error::InvalidInput {
                                reason: format!("Invalid multiasset quantity: {}", e),
                            }
                        })?);
                    }
                }
            }
        }
    }

    // Strict per-asset conservation
    // Lovelace: inputs == outputs + fee (no tolerance)
    let in_lovelace = input_totals.get("lovelace").copied().unwrap_or(0);
    let out_lovelace = output_totals.get("lovelace").copied().unwrap_or(0);

    if in_lovelace < fee_u128 {
        return Err(Error::InvalidInput {
            reason: format!(
                "Insufficient lovelace: inputs {} < fee {}",
                in_lovelace, fee_u128
            ),
        });
    }

    if in_lovelace != out_lovelace.saturating_add(fee_u128) {
        return Err(Error::InvalidInput {
            reason: format!(
                "Lovelace imbalance: inputs {}, outputs {}, fee {}, expected outputs {}",
                in_lovelace,
                out_lovelace,
                fee_u128,
                in_lovelace.saturating_sub(fee_u128)
            ),
        });
    }

    // Other assets: exact equality
    for (unit, in_qty) in input_totals.iter() {
        if unit == "lovelace" {
            continue;
        }
        let out_qty = output_totals.get(unit).copied().unwrap_or(0);
        if *in_qty != out_qty {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Asset imbalance for {}: inputs {}, outputs {}",
                    unit, in_qty, out_qty
                ),
            });
        }
    }

    // Also ensure no extra assets are minted/appearing (including lovelace)
    for (unit, out_qty) in output_totals.iter() {
        let in_qty = input_totals.get(unit).copied().unwrap_or(0);
        if *out_qty > in_qty && unit != "lovelace" {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Outputs create extra asset {}: outputs {}, inputs {}",
                    unit, out_qty, in_qty
                ),
            });
        }
        if unit == "lovelace" && *out_qty > in_lovelace {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Outputs create extra lovelace: outputs {}, inputs {}",
                    out_qty, in_lovelace
                ),
            });
        }
    }

    Ok(())
}

/// Validate withdrawal intent binding using auxiliary metadata.
///
/// Expect a metadata entry with label 1914 containing a map:
/// { "network": "<network>", "tx_body_hash": "<hex blake2b-256(body)>" }
fn validate_withdraw_intent_metadata_with_parsed_tx(
    parsed_tx: &ParsedWithdrawalTx,
    network: &str,
) -> Result<(), Error> {
    validate_withdraw_intent_metadata_from_tx(&parsed_tx.tx, network)
}

#[cfg(test)]
fn validate_withdraw_intent_metadata(tx_cbor: &[u8], network: &str) -> Result<(), Error> {
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec()).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction CBOR: {}", e),
    })?;

    validate_withdraw_intent_metadata_from_tx(&tx, network)
}

fn validate_withdraw_intent_metadata_from_tx(
    tx: &csl::Transaction,
    network: &str,
) -> Result<(), Error> {
    let expected_network = CardanoNetwork::parse(network).map_err(|e| Error::InvalidInput {
        reason: e.to_string(),
    })?;

    let aux = tx.auxiliary_data().ok_or_else(|| Error::InvalidInput {
        reason: "Transaction missing auxiliary data for intent binding".to_string(),
    })?;

    let metadata = aux.metadata().ok_or_else(|| Error::InvalidInput {
        reason: "Auxiliary data missing metadata map".to_string(),
    })?;

    let label = csl::BigNum::from_str("1914").map_err(|e| Error::InvalidInput {
        reason: format!("Invalid metadatum label: {}", e),
    })?;
    let metadatum = metadata.get(&label).ok_or_else(|| Error::InvalidInput {
        reason: "Metadata label 1914 missing for intent binding".to_string(),
    })?;

    let map = metadatum.as_map().map_err(|e| Error::InvalidInput {
        reason: format!("Metadata label 1914 must be a map: {}", e),
    })?;

    let mut network_ok = false;
    let mut hash_ok = false;

    let keys = map.keys();
    for i in 0..keys.len() {
        let key_md = keys.get(i);
        let key_txt = match key_md.as_text() {
            Ok(t) => t,
            Err(_) => continue,
        };

        let val = map.get(&key_md).map_err(|e| Error::InvalidInput {
            reason: format!("Metadata map lookup failed: {}", e),
        })?;

        match key_txt.as_str() {
            "network" => {
                if let Ok(n_txt) = val.as_text() {
                    network_ok = CardanoNetwork::parse(&n_txt).ok() == Some(expected_network);
                }
            }
            "tx_body_hash" => {
                if let Ok(h_txt) = val.as_text() {
                    // Compute body hash
                    let body_bytes = tx.body().to_bytes();
                    type Blake2b256 = blake2::Blake2b<blake2::digest::consts::U32>;
                    let h = Blake2b256::digest(&body_bytes);
                    let mut h_arr = [0u8; 32];
                    h_arr.copy_from_slice(&h);
                    let expected_hex = hex::encode(h_arr);
                    hash_ok = h_txt.eq_ignore_ascii_case(&expected_hex);
                }
            }
            _ => {}
        }
    }

    if !network_ok {
        return Err(Error::InvalidInput {
            reason: "Intent metadata network mismatch".to_string(),
        });
    }
    if !hash_ok {
        return Err(Error::InvalidInput {
            reason: "Intent metadata tx_body_hash mismatch".to_string(),
        });
    }

    Ok(())
}

/// Validate that outputs stay on the same network and do not return change to the script.
fn validate_network_and_change_outputs_with_parsed_tx(
    parsed_tx: &ParsedWithdrawalTx,
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<(), Error> {
    validate_network_and_change_outputs_from_tx(&parsed_tx.tx, wallet)
}

#[cfg(test)]
fn validate_network_and_change_outputs(
    tx_cbor: &[u8],
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<(), Error> {
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec()).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction CBOR: {}", e),
    })?;

    validate_network_and_change_outputs_from_tx(&tx, wallet)
}

fn validate_network_and_change_outputs_from_tx(
    tx: &csl::Transaction,
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<(), Error> {
    let expected_network_id = CardanoNetwork::parse(&wallet.network)
        .map_err(|e| Error::InvalidInput {
            reason: e.to_string(),
        })?
        .address_network_id();

    for (idx, output) in (&tx.body().outputs()).into_iter().enumerate() {
        let addr = output.address();

        // Network guard
        let net_id = addr.network_id().map_err(|e| Error::InvalidInput {
            reason: format!("Failed to read network id for output {}: {}", idx, e),
        })?;
        if net_id != expected_network_id {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Output {} has network_id {} but wallet is {}",
                    idx, net_id, wallet.network
                ),
            });
        }

        // Reject change back to the script until change notes are implemented
        let bech32 = addr.to_bech32(None).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid output address: {}", e),
        })?;

        if bech32 == wallet.script_address {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Output {} pays back to script address (change). Change notes not yet supported.",
                    idx
                ),
            });
        }
    }

    Ok(())
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
    use pallas_codec::minicbor;
    use pallas_primitives::{
        BoundedBytes,
        Constr,
        MaybeIndefArray,
        alonzo::PlutusData,
    };
    use serde_json::json;
    use tempfile::TempDir;

    use super::*;
    use crate::{
        cardano::generate_payment_keypair,
        config::Config,
        database::{CARDANO_WALLET, DEPOSITS, NOTES, WITHDRAWALS, Database},
        routes::Context,
    };

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

    fn insert_wallet(ctx: &Context, payment_sk: Vec<u8>, payment_vk: Vec<u8>, script_address: &str) {
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

    fn seed_deposit(ctx: &Context, utxo_ref: mugraph_core::types::UtxoRef, intent_hash: [u8; 32]) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let record = mugraph_core::types::DepositRecord::with_intent_hash(90, now, now + 3600, intent_hash);
        seed_deposit_record(ctx, utxo_ref, record);
    }

    fn seed_deposit_record(
        ctx: &Context,
        utxo_ref: mugraph_core::types::UtxoRef,
        record: mugraph_core::types::DepositRecord,
    ) {
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
            table.insert(&withdrawal_key_from_hex(tx_hash), &record).unwrap();
        }
        write_tx.commit().unwrap();
    }

    fn build_datum_cbor_hex(user_hash: Vec<u8>, node_hash: Vec<u8>, intent_hash: Vec<u8>) -> String {
        let datum = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: MaybeIndefArray::Def(vec![
                PlutusData::BoundedBytes(BoundedBytes::from(user_hash)),
                PlutusData::BoundedBytes(BoundedBytes::from(node_hash)),
                PlutusData::BoundedBytes(BoundedBytes::from(intent_hash)),
            ]),
        });

        hex::encode(minicbor::to_vec(&datum).unwrap())
    }

    fn build_withdraw_request(
        user_sk: &SigningKey,
        input_tx_hash: [u8; 32],
        input_value: u64,
        output_value: u64,
        fee: u64,
        network: &str,
    ) -> WithdrawRequest {
        build_withdraw_request_with_balance_check(
            user_sk,
            input_tx_hash,
            input_value,
            output_value,
            fee,
            network,
            true,
        )
    }

    fn build_withdraw_request_with_balance_check(
        user_sk: &SigningKey,
        input_tx_hash: [u8; 32],
        input_value: u64,
        output_value: u64,
        fee: u64,
        network: &str,
        assert_balanced: bool,
    ) -> WithdrawRequest {
        let tx_hash = csl::TransactionHash::from_bytes(input_tx_hash.to_vec()).unwrap();
        let input = csl::TransactionInput::new(&tx_hash, 0);
        let mut inputs = csl::TransactionInputs::new();
        inputs.add(&input);

        let addr = csl::Address::from_bech32(
            "addr_test1vru4e2un2tq50q4rv6qzk7t8w34gjdtw3y2uzuqxzj0ldrqqactxh",
        )
        .unwrap();
        let output_coin = csl::Coin::from_str(&output_value.to_string()).unwrap();
        let value = csl::Value::new(&output_coin);
        let output = csl::TransactionOutput::new(&addr, &value);
        let mut outputs = csl::TransactionOutputs::new();
        outputs.add(&output);

        let fee_coin = csl::Coin::from_str(&fee.to_string()).unwrap();
        let mut body = csl::TransactionBody::new_tx_body(&inputs, &outputs, &fee_coin);

        let pk = user_sk.verifying_key();
        let pk_hash = csl::PublicKey::from_bytes(pk.as_bytes()).unwrap().hash();
        let mut required = csl::Ed25519KeyHashes::new();
        required.add(&pk_hash);
        body.set_required_signers(&required);

        type Blake2b256 = blake2::Blake2b<blake2::digest::consts::U32>;
        let body_hash = Blake2b256::digest(body.to_bytes());
        let tx_body_hash_hex = hex::encode(body_hash);

        let mut metadata_map = csl::MetadataMap::new();
        metadata_map
            .insert_str(
                "network",
                &csl::TransactionMetadatum::new_text(network.to_string()).unwrap(),
            )
            .unwrap();
        metadata_map
            .insert_str(
                "tx_body_hash",
                &csl::TransactionMetadatum::new_text(tx_body_hash_hex).unwrap(),
            )
            .unwrap();
        let metadatum = csl::TransactionMetadatum::new_map(&metadata_map);
        let mut general_md = csl::GeneralTransactionMetadata::new();
        general_md.insert(&csl::BigNum::from_str("1914").unwrap(), &metadatum);
        let mut aux = csl::AuxiliaryData::new();
        aux.set_metadata(&general_md);

        let tx_hash_csl = csl::TransactionHash::from_bytes(body_hash.to_vec()).unwrap();
        let private = csl::PrivateKey::from_normal_bytes(user_sk.as_bytes()).unwrap();
        let witness = csl::make_vkey_witness(&tx_hash_csl, &private);
        let mut witness_set = csl::TransactionWitnessSet::new();
        let mut vkeys = csl::Vkeywitnesses::new();
        vkeys.add(&witness);
        witness_set.set_vkeys(&vkeys);

        let tx = csl::Transaction::new(&body, &witness_set, Some(aux));
        let tx_cbor = tx.to_bytes();
        let tx_hash = hex::encode(compute_tx_hash(&tx_cbor).unwrap());

        if assert_balanced {
            assert_eq!(input_value, output_value + fee, "test transaction must balance");
        }

        WithdrawRequest {
            notes: vec![BlindSignature {
                signature: mugraph_core::types::Blinded(mugraph_core::types::Signature::from([9u8; 32])),
                proof: Default::default(),
            }],
            tx_cbor: hex::encode(tx_cbor),
            tx_hash,
        }
    }

    fn build_withdraw_request_without_inputs(
        user_sk: &SigningKey,
        output_value: u64,
        fee: u64,
        network: &str,
    ) -> WithdrawRequest {
        let inputs = csl::TransactionInputs::new();
        let addr = csl::Address::from_bech32(
            "addr_test1vru4e2un2tq50q4rv6qzk7t8w34gjdtw3y2uzuqxzj0ldrqqactxh",
        )
        .unwrap();
        let output_coin = csl::Coin::from_str(&output_value.to_string()).unwrap();
        let value = csl::Value::new(&output_coin);
        let output = csl::TransactionOutput::new(&addr, &value);
        let mut outputs = csl::TransactionOutputs::new();
        outputs.add(&output);

        let fee_coin = csl::Coin::from_str(&fee.to_string()).unwrap();
        let mut body = csl::TransactionBody::new_tx_body(&inputs, &outputs, &fee_coin);

        let pk = user_sk.verifying_key();
        let pk_hash = csl::PublicKey::from_bytes(pk.as_bytes()).unwrap().hash();
        let mut required = csl::Ed25519KeyHashes::new();
        required.add(&pk_hash);
        body.set_required_signers(&required);

        type Blake2b256 = blake2::Blake2b<blake2::digest::consts::U32>;
        let body_hash = Blake2b256::digest(body.to_bytes());
        let tx_body_hash_hex = hex::encode(body_hash);

        let mut metadata_map = csl::MetadataMap::new();
        metadata_map
            .insert_str(
                "network",
                &csl::TransactionMetadatum::new_text(network.to_string()).unwrap(),
            )
            .unwrap();
        metadata_map
            .insert_str(
                "tx_body_hash",
                &csl::TransactionMetadatum::new_text(tx_body_hash_hex).unwrap(),
            )
            .unwrap();
        let metadatum = csl::TransactionMetadatum::new_map(&metadata_map);
        let mut general_md = csl::GeneralTransactionMetadata::new();
        general_md.insert(&csl::BigNum::from_str("1914").unwrap(), &metadatum);
        let mut aux = csl::AuxiliaryData::new();
        aux.set_metadata(&general_md);

        let tx_hash_csl = csl::TransactionHash::from_bytes(body_hash.to_vec()).unwrap();
        let private = csl::PrivateKey::from_normal_bytes(user_sk.as_bytes()).unwrap();
        let witness = csl::make_vkey_witness(&tx_hash_csl, &private);
        let mut witness_set = csl::TransactionWitnessSet::new();
        let mut vkeys = csl::Vkeywitnesses::new();
        vkeys.add(&witness);
        witness_set.set_vkeys(&vkeys);

        let tx = csl::Transaction::new(&body, &witness_set, Some(aux));
        let tx_cbor = tx.to_bytes();

        WithdrawRequest {
            notes: vec![BlindSignature {
                signature: mugraph_core::types::Blinded(
                    mugraph_core::types::Signature::from([9u8; 32]),
                ),
                proof: Default::default(),
            }],
            tx_hash: hex::encode(compute_tx_hash(&tx_cbor).unwrap()),
            tx_cbor: hex::encode(tx_cbor),
        }
    }

    fn withdrawal_key_from_hex(tx_hash: &str) -> mugraph_core::types::WithdrawalKey {
        let bytes = hex::decode(tx_hash).unwrap();
        let array: [u8; 32] = bytes.try_into().unwrap();
        mugraph_core::types::WithdrawalKey::new(0, array)
    }

    async fn spawn_withdraw_provider_mock(
        script_address: String,
        datum_cbor_hex: String,
        input_value: u64,
        submit_status: StatusCode,
        submit_hash: String,
    ) -> String {
        async fn tx_info() -> impl IntoResponse {
            (StatusCode::OK, axum::Json(json!({"block_height": 90})))
        }

        async fn tx_utxos(
            Path(tx_hash): Path<String>,
            axum::extract::State(state): axum::extract::State<(String, String, u64, StatusCode, String)>,
        ) -> impl IntoResponse {
            let (script_address, _datum_hex, input_value, _submit_status, _submit_hash) = state;
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
            axum::extract::State(state): axum::extract::State<(String, String, u64, StatusCode, String)>,
        ) -> impl IntoResponse {
            let (_script_address, datum_hex, _input_value, _submit_status, _submit_hash) = state;
            (StatusCode::OK, axum::Json(json!({"cbor": datum_hex})))
        }

        async fn submit(
            axum::extract::State(state): axum::extract::State<(String, String, u64, StatusCode, String)>,
        ) -> impl IntoResponse {
            let (_script_address, _datum_hex, _input_value, submit_status, submit_hash) = state;
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
            .with_state((script_address, datum_cbor_hex, input_value, submit_status, submit_hash));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
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

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
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

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        format!("http://{addr}")
    }

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

        assert!(validate_transaction_balance(&tx_cbor, &totals, max_fee).is_ok());

        // Fee too high
        let max_fee = 500_000;
        assert!(validate_transaction_balance(&tx_cbor, &totals, max_fee).is_err());
    }

    #[test]
    fn test_validate_transaction_balance_respects_fee_tolerance() {
        let tx = minimal_tx_with_values(1_000_000, 1_050_000); // fee within 5% of 1_000_000
        let tx_cbor = tx.to_bytes();
        let mut totals = HashMap::new();
        totals.insert("lovelace".to_string(), 2_050_000u128);

        let res = validate_transaction_balance_with_tolerance(&tx_cbor, &totals, 1_000_000, 5);
        assert!(res.is_ok());
    }

    /// required_signers present but missing matching witness => reject
    #[tokio::test]
    async fn test_required_signer_missing_witness() {
        let sk = SigningKey::from_bytes(&[1u8; 32]);
        let pk = sk.verifying_key();
        let pk_hash = csl::PublicKey::from_bytes(pk.as_bytes())
            .unwrap()
            .hash()
            .to_hex();
        let mut expected = HashSet::new();
        expected.insert(pk_hash.clone());

        let tx = minimal_tx_with_required_signer(&pk_hash, None);
        let notes: Vec<BlindSignature> = vec![BlindSignature::default()];
        let wallet = mugraph_core::types::CardanoWallet::new(
            vec![],
            vec![],
            vec![],
            vec![],
            "addr_test...".to_string(),
            "preprod".to_string(),
        );

        let res = validate_user_witnesses(&tx.to_bytes(), &notes, &expected, &wallet).await;
        assert!(res.is_err());
    }

    /// required_signers present and matching witness => ok
    #[tokio::test]
    async fn test_required_signer_with_witness() {
        let sk = SigningKey::from_bytes(&[2u8; 32]);
        let pk = sk.verifying_key();
        let pk_csl = csl::PublicKey::from_bytes(pk.as_bytes()).unwrap();
        let pk_hash = pk_csl.hash().to_hex();
        let mut expected = HashSet::new();
        expected.insert(pk_hash.clone());

        let tx_body_only = minimal_tx_with_required_signer(&pk_hash, None).body();
        let tx_hash_csl = tx_hash_from_body(&tx_body_only);
        let private = csl::PrivateKey::from_normal_bytes(sk.as_bytes()).unwrap();
        let vkey_witness = csl::make_vkey_witness(&tx_hash_csl, &private);

        let mut witness_set = csl::TransactionWitnessSet::new();
        let mut vkeys = csl::Vkeywitnesses::new();
        vkeys.add(&vkey_witness);
        witness_set.set_vkeys(&vkeys);

        let tx = csl::Transaction::new(&tx_body_only, &witness_set, None);

        let notes: Vec<BlindSignature> = vec![BlindSignature::default()];
        let wallet = mugraph_core::types::CardanoWallet::new(
            vec![],
            vec![],
            vec![],
            vec![],
            "addr_test...".to_string(),
            "preprod".to_string(),
        );

        let res = validate_user_witnesses(&tx.to_bytes(), &notes, &expected, &wallet).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_multi_owner_missing_from_required_signers() {
        let sk1 = SigningKey::from_bytes(&[3u8; 32]);
        let sk2 = SigningKey::from_bytes(&[4u8; 32]);
        let pk1_hash = csl::PublicKey::from_bytes(sk1.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_hex();
        let pk2_hash = csl::PublicKey::from_bytes(sk2.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_hex();

        let expected = HashSet::from([pk1_hash.clone(), pk2_hash.clone()]);
        let tx_body =
            minimal_tx_body_with_required_signers(std::slice::from_ref(&pk1_hash));
        let tx_hash_csl = tx_hash_from_body(&tx_body);
        let witness_set = witness_set_with_vkey_signers(&tx_hash_csl, &[&sk1]);
        let tx = csl::Transaction::new(&tx_body, &witness_set, None);

        let notes: Vec<BlindSignature> = vec![BlindSignature::default(), BlindSignature::default()];
        let wallet = mugraph_core::types::CardanoWallet::new(
            vec![],
            vec![],
            vec![],
            vec![],
            "addr_test...".to_string(),
            "preprod".to_string(),
        );

        let err = validate_user_witnesses(&tx.to_bytes(), &notes, &expected, &wallet)
            .await
            .unwrap_err();
        assert!(format!("{err:?}").contains("Required signer set does not include input owner hash"));
    }

    #[tokio::test]
    async fn test_multi_owner_missing_witness_for_required_signer() {
        let sk1 = SigningKey::from_bytes(&[5u8; 32]);
        let sk2 = SigningKey::from_bytes(&[6u8; 32]);
        let pk1_hash = csl::PublicKey::from_bytes(sk1.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_hex();
        let pk2_hash = csl::PublicKey::from_bytes(sk2.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_hex();

        let expected = HashSet::from([pk1_hash.clone(), pk2_hash.clone()]);
        let tx_body = minimal_tx_body_with_required_signers(&[pk1_hash.clone(), pk2_hash.clone()]);
        let tx_hash_csl = tx_hash_from_body(&tx_body);
        let witness_set = witness_set_with_vkey_signers(&tx_hash_csl, &[&sk1]);
        let tx = csl::Transaction::new(&tx_body, &witness_set, None);

        let notes: Vec<BlindSignature> = vec![BlindSignature::default(), BlindSignature::default()];
        let wallet = mugraph_core::types::CardanoWallet::new(
            vec![],
            vec![],
            vec![],
            vec![],
            "addr_test...".to_string(),
            "preprod".to_string(),
        );

        let err = validate_user_witnesses(&tx.to_bytes(), &notes, &expected, &wallet)
            .await
            .unwrap_err();
        assert!(format!("{err:?}").contains("Missing witnesses for required_signers"));
    }

    #[tokio::test]
    async fn test_bootstrap_witness_counts_when_valid() {
        let key = csl::Bip32PrivateKey::generate_ed25519_bip32().unwrap();
        let byron_address = csl::ByronAddress::from_base58(
            "Ae2tdPwUPEZ5uzkzh1o2DHECiUi3iugvnnKHRisPgRRP3CTF4KCMvy54Xd3",
        )
        .unwrap();
        let required_hash = key.to_raw_key().to_public().hash().to_hex();
        let expected = HashSet::from([required_hash.clone()]);
        let tx_body = minimal_tx_body_with_required_signers(&[required_hash]);
        let tx_hash_csl = tx_hash_from_body(&tx_body);
        let bootstrap = csl::make_icarus_bootstrap_witness(&tx_hash_csl, &byron_address, &key);

        let mut witness_set = csl::TransactionWitnessSet::new();
        let mut bootstraps = csl::BootstrapWitnesses::new();
        bootstraps.add(&bootstrap);
        witness_set.set_bootstraps(&bootstraps);

        let tx = csl::Transaction::new(&tx_body, &witness_set, None);
        let notes: Vec<BlindSignature> = vec![BlindSignature::default()];
        let wallet = mugraph_core::types::CardanoWallet::new(
            vec![],
            vec![],
            vec![],
            vec![],
            "addr_test...".to_string(),
            "preprod".to_string(),
        );

        let res = validate_user_witnesses(&tx.to_bytes(), &notes, &expected, &wallet).await;
        assert!(res.is_ok());
    }

    #[test]
    fn test_intent_metadata_binding() {
        // Build tx body
        let tx_hash = csl::TransactionHash::from_bytes(vec![0; 32]).unwrap();
        let input = csl::TransactionInput::new(&tx_hash, 0);
        let mut inputs = csl::TransactionInputs::new();
        inputs.add(&input);

        let addr = csl::Address::from_bech32(
            "addr_test1vru4e2un2tq50q4rv6qzk7t8w34gjdtw3y2uzuqxzj0ldrqqactxh",
        )
        .unwrap();
        let coin = csl::Coin::from_str("1000000").unwrap();
        let value = csl::Value::new(&coin);
        let output = csl::TransactionOutput::new(&addr, &value);
        let mut outputs = csl::TransactionOutputs::new();
        outputs.add(&output);

        let fee = csl::Coin::from_str("170000").unwrap();
        let body = csl::TransactionBody::new_tx_body(&inputs, &outputs, &fee);

        // Compute body hash
        let body_bytes = body.to_bytes();
        type Blake2b256 = blake2::Blake2b<blake2::digest::consts::U32>;
        let h = Blake2b256::digest(&body_bytes);
        let mut h_arr = [0u8; 32];
        h_arr.copy_from_slice(&h);
        let h_hex = hex::encode(h_arr);

        // Build metadata label 1914 with network + tx_body_hash
        let mut md_map = csl::MetadataMap::new();
        let md_network = csl::TransactionMetadatum::new_text("preprod".to_string()).unwrap();
        let md_hash = csl::TransactionMetadatum::new_text(h_hex.clone()).unwrap();
        md_map.insert_str("network", &md_network).unwrap();
        md_map.insert_str("tx_body_hash", &md_hash).unwrap();
        let metadatum = csl::TransactionMetadatum::new_map(&md_map);
        let mut general_md = csl::GeneralTransactionMetadata::new();
        general_md.insert(&csl::BigNum::from_str("1914").unwrap(), &metadatum);
        let mut aux = csl::AuxiliaryData::new();
        aux.set_metadata(&general_md);

        let witness_set = csl::TransactionWitnessSet::new();
        let tx = csl::Transaction::new(&body, &witness_set, Some(aux));

        assert!(validate_withdraw_intent_metadata(&tx.to_bytes(), "preprod").is_ok());

        // Tamper network
        assert!(validate_withdraw_intent_metadata(&tx.to_bytes(), "mainnet").is_err());
    }

    /// Reject outputs that pay back to the script address (change not supported yet)
    #[test]
    fn test_reject_change_output_to_script() {
        // Build tx with output to script address
        let tx_hash = csl::TransactionHash::from_bytes(vec![0; 32]).unwrap();
        let input = csl::TransactionInput::new(&tx_hash, 0);
        let mut inputs = csl::TransactionInputs::new();
        inputs.add(&input);

        // Build a valid testnet enterprise address to reuse as script address
        let key_hash = csl::Ed25519KeyHash::from_bytes(vec![1u8; 28]).unwrap();
        let cred = csl::Credential::from_keyhash(&key_hash);
        let addr = csl::EnterpriseAddress::new(0, &cred).to_address();
        let script_addr = addr.to_bech32(None).unwrap();

        let coin = csl::Coin::from_str("1000000").unwrap();
        let value = csl::Value::new(&coin);
        let output = csl::TransactionOutput::new(&addr, &value);
        let mut outputs = csl::TransactionOutputs::new();
        outputs.add(&output);

        let fee = csl::Coin::from_str("170000").unwrap();
        let body = csl::TransactionBody::new_tx_body(&inputs, &outputs, &fee);
        let witness_set = csl::TransactionWitnessSet::new();
        let tx = csl::Transaction::new(&body, &witness_set, None);

        let wallet = mugraph_core::types::CardanoWallet::new(
            vec![],
            vec![],
            vec![],
            vec![],
            script_addr.to_string(),
            "preprod".to_string(),
        );

        let err = validate_network_and_change_outputs(&tx.to_bytes(), &wallet).unwrap_err();
        assert!(format!("{:?}", err).contains("Change notes not yet supported"));
    }

    #[test]
    fn test_change_notes_are_not_fabricated_for_non_script_outputs() {
        let tx = minimal_tx_with_values(1_000_000, 170_000);
        let wallet = mugraph_core::types::CardanoWallet::new(
            vec![],
            vec![],
            vec![],
            vec![],
            "addr_test1different_script_address".to_string(),
            "preprod".to_string(),
        );
        let sk = mugraph_core::types::SecretKey::from([7u8; 32]);
        let keypair = mugraph_core::types::Keypair {
            public_key: sk.public(),
            secret_key: sk,
        };

        let notes = calculate_change_notes(
            &WithdrawRequest {
                notes: vec![],
                tx_cbor: String::new(),
                tx_hash: String::new(),
            },
            &tx.to_bytes(),
            &wallet,
            &keypair,
        )
        .unwrap();

        assert!(notes.is_empty());
    }

    /// Reject outputs on wrong network
    #[test]
    fn test_reject_output_wrong_network() {
        // Build tx with mainnet address while wallet is preprod
        let tx_hash = csl::TransactionHash::from_bytes(vec![0; 32]).unwrap();
        let input = csl::TransactionInput::new(&tx_hash, 0);
        let mut inputs = csl::TransactionInputs::new();
        inputs.add(&input);

        // mainnet enterprise address (network id 1)
        let key_hash = csl::Ed25519KeyHash::from_bytes(vec![2u8; 28]).unwrap();
        let cred = csl::Credential::from_keyhash(&key_hash);
        let addr = csl::EnterpriseAddress::new(1, &cred).to_address();
        let coin = csl::Coin::from_str("1000000").unwrap();
        let value = csl::Value::new(&coin);
        let output = csl::TransactionOutput::new(&addr, &value);
        let mut outputs = csl::TransactionOutputs::new();
        outputs.add(&output);

        let fee = csl::Coin::from_str("170000").unwrap();
        let body = csl::TransactionBody::new_tx_body(&inputs, &outputs, &fee);
        let witness_set = csl::TransactionWitnessSet::new();
        let tx = csl::Transaction::new(&body, &witness_set, None);

        let wallet = mugraph_core::types::CardanoWallet::new(
            vec![],
            vec![],
            vec![],
            vec![],
            {
                let key_hash = csl::Ed25519KeyHash::from_bytes(vec![3u8; 28]).unwrap();
                let cred = csl::Credential::from_keyhash(&key_hash);
                csl::EnterpriseAddress::new(0, &cred)
                    .to_address()
                    .to_bech32(None)
                    .unwrap()
            },
            "preprod".to_string(),
        );

        let err = validate_network_and_change_outputs(&tx.to_bytes(), &wallet).unwrap_err();
        assert!(format!("{:?}", err).contains("network_id 1"));
    }

    #[tokio::test]
    async fn handle_withdraw_happy_path_marks_withdrawal_completed_and_spends_deposit() {
        let user_sk = SigningKey::from_bytes(&[3u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let input_tx_hash = [0xabu8; 32];
        let input_value = 1_170_000u64;
        let request = build_withdraw_request(&user_sk, input_tx_hash, input_value, 1_000_000, 170_000, "preprod");

        let node_hash = csl::PublicKey::from_bytes(&payment_vk).unwrap().hash().to_bytes();
        let user_hash = csl::PublicKey::from_bytes(user_sk.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_bytes();
        let datum_hex = build_datum_cbor_hex(user_hash, node_hash, vec![0u8; 32]);
        let provider_url = spawn_withdraw_provider_mock(
            "addr_test1script".to_string(),
            datum_hex,
            input_value,
            StatusCode::OK,
            request.tx_hash.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");
        seed_deposit(&ctx, mugraph_core::types::UtxoRef::new(input_tx_hash, 0), [0u8; 32]);

        let response = handle_withdraw(&request, &ctx).await.expect("withdraw accepted");
        assert!(matches!(response, Response::Withdraw { .. }));

        let read_tx = ctx.database.read().unwrap();
        let notes = read_tx.open_table(NOTES).unwrap();
        let note_signature = mugraph_core::types::Signature::from([9u8; 32]);
        assert!(notes.get(note_signature).unwrap().is_some());

        let withdrawals = read_tx.open_table(WITHDRAWALS).unwrap();
        let key = withdrawal_key_from_hex(&request.tx_hash);
        assert_eq!(
            withdrawals.get(&key).unwrap().unwrap().value().status,
            mugraph_core::types::WithdrawalStatus::Completed
        );

        let deposits = read_tx.open_table(DEPOSITS).unwrap();
        assert!(deposits
            .get(mugraph_core::types::UtxoRef::new(input_tx_hash, 0))
            .unwrap()
            .unwrap()
            .value()
            .spent);
    }

    fn assert_preflight_rejection_leaves_state_untouched(ctx: &Context, tx_hash: &str) {
        let read_tx = ctx.database.read().unwrap();
        let notes = read_tx.open_table(NOTES).unwrap();
        let note_signature = mugraph_core::types::Signature::from([9u8; 32]);
        assert!(notes.get(note_signature).unwrap().is_none());

        let withdrawals = read_tx.open_table(WITHDRAWALS).unwrap();
        let key = withdrawal_key_from_hex(tx_hash);
        assert!(withdrawals.get(&key).unwrap().is_none());
    }

    #[tokio::test]
    async fn handle_withdraw_hash_mismatch_does_not_mutate_notes_or_withdrawals() {
        let user_sk = SigningKey::from_bytes(&[13u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let request = build_withdraw_request(
            &user_sk,
            [0xadu8; 32],
            1_170_000,
            1_000_000,
            170_000,
            "preprod",
        );

        let ctx = test_context_with_provider_url(Some("http://127.0.0.1:1".to_string()));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");

        let mut mismatched = request.clone();
        mismatched.tx_hash = "ff".repeat(32);

        let err = handle_withdraw(&mismatched, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("Transaction hash mismatch"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &mismatched.tx_hash);
    }

    #[tokio::test]
    async fn handle_withdraw_metadata_mismatch_does_not_mutate_notes_or_withdrawals() {
        let user_sk = SigningKey::from_bytes(&[14u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let input_tx_hash = [0xbdu8; 32];
        let input_value = 1_170_000u64;
        let request = build_withdraw_request(
            &user_sk,
            input_tx_hash,
            input_value,
            1_000_000,
            170_000,
            "mainnet",
        );

        let node_hash = csl::PublicKey::from_bytes(&payment_vk).unwrap().hash().to_bytes();
        let user_hash = csl::PublicKey::from_bytes(user_sk.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_bytes();
        let datum_hex = build_datum_cbor_hex(user_hash, node_hash, vec![0u8; 32]);
        let provider_url = spawn_withdraw_provider_mock(
            "addr_test1script".to_string(),
            datum_hex,
            input_value,
            StatusCode::OK,
            request.tx_hash.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");
        seed_deposit(&ctx, mugraph_core::types::UtxoRef::new(input_tx_hash, 0), [0u8; 32]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("network mismatch"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request.tx_hash);
    }

    #[tokio::test]
    async fn handle_withdraw_balance_failure_does_not_mutate_notes_or_withdrawals() {
        let user_sk = SigningKey::from_bytes(&[15u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let input_tx_hash = [0xbeu8; 32];
        let input_value = 1_170_000u64;
        let request = build_withdraw_request_with_balance_check(
            &user_sk,
            input_tx_hash,
            input_value,
            1_100_000,
            170_000,
            "preprod",
            false,
        );

        let node_hash = csl::PublicKey::from_bytes(&payment_vk).unwrap().hash().to_bytes();
        let user_hash = csl::PublicKey::from_bytes(user_sk.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_bytes();
        let datum_hex = build_datum_cbor_hex(user_hash, node_hash, vec![0u8; 32]);
        let provider_url = spawn_withdraw_provider_mock(
            "addr_test1script".to_string(),
            datum_hex,
            input_value,
            StatusCode::OK,
            request.tx_hash.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");
        seed_deposit(&ctx, mugraph_core::types::UtxoRef::new(input_tx_hash, 0), [0u8; 32]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("Lovelace imbalance"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request.tx_hash);
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_already_spent_deposit_without_mutating_state() {
        let user_sk = SigningKey::from_bytes(&[16u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let input_tx_hash = [0xbfu8; 32];
        let input_value = 1_170_000u64;
        let request = build_withdraw_request(
            &user_sk,
            input_tx_hash,
            input_value,
            1_000_000,
            170_000,
            "preprod",
        );

        let node_hash = csl::PublicKey::from_bytes(&payment_vk).unwrap().hash().to_bytes();
        let user_hash = csl::PublicKey::from_bytes(user_sk.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_bytes();
        let datum_hex = build_datum_cbor_hex(user_hash, node_hash, vec![0u8; 32]);
        let provider_url = spawn_withdraw_provider_mock(
            "addr_test1script".to_string(),
            datum_hex,
            input_value,
            StatusCode::OK,
            request.tx_hash.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut record = mugraph_core::types::DepositRecord::with_intent_hash(
            90,
            now,
            now + 3600,
            [0u8; 32],
        );
        record.spent = true;
        seed_deposit_record(
            &ctx,
            mugraph_core::types::UtxoRef::new(input_tx_hash, 0),
            record,
        );

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("deposit already spent"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request.tx_hash);
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_expired_deposit_without_mutating_state() {
        let user_sk = SigningKey::from_bytes(&[17u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let input_tx_hash = [0xc0u8; 32];
        let input_value = 1_170_000u64;
        let request = build_withdraw_request(
            &user_sk,
            input_tx_hash,
            input_value,
            1_000_000,
            170_000,
            "preprod",
        );

        let node_hash = csl::PublicKey::from_bytes(&payment_vk).unwrap().hash().to_bytes();
        let user_hash = csl::PublicKey::from_bytes(user_sk.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_bytes();
        let datum_hex = build_datum_cbor_hex(user_hash, node_hash, vec![0u8; 32]);
        let provider_url = spawn_withdraw_provider_mock(
            "addr_test1script".to_string(),
            datum_hex,
            input_value,
            StatusCode::OK,
            request.tx_hash.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let record = mugraph_core::types::DepositRecord::with_intent_hash(
            90,
            now.saturating_sub(3600),
            now.saturating_sub(1),
            [0u8; 32],
        );
        seed_deposit_record(
            &ctx,
            mugraph_core::types::UtxoRef::new(input_tx_hash, 0),
            record,
        );

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("deposit expired"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request.tx_hash);
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_intent_hash_mismatch_without_mutating_state() {
        let user_sk = SigningKey::from_bytes(&[18u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let input_tx_hash = [0xc1u8; 32];
        let input_value = 1_170_000u64;
        let request = build_withdraw_request(
            &user_sk,
            input_tx_hash,
            input_value,
            1_000_000,
            170_000,
            "preprod",
        );

        let node_hash = csl::PublicKey::from_bytes(&payment_vk).unwrap().hash().to_bytes();
        let user_hash = csl::PublicKey::from_bytes(user_sk.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_bytes();
        let datum_hex = build_datum_cbor_hex(user_hash, node_hash, vec![0u8; 32]);
        let provider_url = spawn_withdraw_provider_mock(
            "addr_test1script".to_string(),
            datum_hex,
            input_value,
            StatusCode::OK,
            request.tx_hash.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");
        seed_deposit(
            &ctx,
            mugraph_core::types::UtxoRef::new(input_tx_hash, 0),
            [7u8; 32],
        );

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("Intent hash mismatch"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request.tx_hash);
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_missing_deposit_record_without_mutating_state() {
        let user_sk = SigningKey::from_bytes(&[19u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let input_tx_hash = [0xc2u8; 32];
        let input_value = 1_170_000u64;
        let request = build_withdraw_request(
            &user_sk,
            input_tx_hash,
            input_value,
            1_000_000,
            170_000,
            "preprod",
        );

        let node_hash = csl::PublicKey::from_bytes(&payment_vk).unwrap().hash().to_bytes();
        let user_hash = csl::PublicKey::from_bytes(user_sk.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_bytes();
        let datum_hex = build_datum_cbor_hex(user_hash, node_hash, vec![0u8; 32]);
        let provider_url = spawn_withdraw_provider_mock(
            "addr_test1script".to_string(),
            datum_hex,
            input_value,
            StatusCode::OK,
            request.tx_hash.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("deposit not found"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request.tx_hash);
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_transactions_without_inputs_before_state_mutation() {
        let user_sk = SigningKey::from_bytes(&[21u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let request = build_withdraw_request_without_inputs(&user_sk, 1_000_000, 170_000, "preprod");

        let ctx = test_context_with_provider_url(Some("http://127.0.0.1:1".to_string()));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("No inputs found in transaction"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request.tx_hash);
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_inputs_not_from_script_address() {
        let user_sk = SigningKey::from_bytes(&[22u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let input_tx_hash = [0xc4u8; 32];
        let input_value = 1_170_000u64;
        let request = build_withdraw_request(
            &user_sk,
            input_tx_hash,
            input_value,
            1_000_000,
            170_000,
            "preprod",
        );

        let node_hash = csl::PublicKey::from_bytes(&payment_vk).unwrap().hash().to_bytes();
        let user_hash = csl::PublicKey::from_bytes(user_sk.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_bytes();
        let datum_hex = build_datum_cbor_hex(user_hash, node_hash, vec![0u8; 32]);
        let provider_url = spawn_withdraw_provider_mock(
            "addr_test1different".to_string(),
            datum_hex,
            input_value,
            StatusCode::OK,
            request.tx_hash.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");
        seed_deposit(&ctx, mugraph_core::types::UtxoRef::new(input_tx_hash, 0), [0u8; 32]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("is not from script address"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request.tx_hash);
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_inputs_missing_inline_datum() {
        let user_sk = SigningKey::from_bytes(&[23u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let input_tx_hash = [0xc5u8; 32];
        let input_value = 1_170_000u64;
        let request = build_withdraw_request(
            &user_sk,
            input_tx_hash,
            input_value,
            1_000_000,
            170_000,
            "preprod",
        );

        let provider_url =
            spawn_withdraw_provider_mock_without_inline_datum("addr_test1script".to_string(), input_value)
                .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");
        seed_deposit(&ctx, mugraph_core::types::UtxoRef::new(input_tx_hash, 0), [0u8; 32]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("missing inline datum"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request.tx_hash);
    }

    #[tokio::test]
    async fn handle_withdraw_rejects_inputs_with_wrong_node_hash() {
        let user_sk = SigningKey::from_bytes(&[24u8; 32]);
        let wrong_node_sk = SigningKey::from_bytes(&[25u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let input_tx_hash = [0xc6u8; 32];
        let input_value = 1_170_000u64;
        let request = build_withdraw_request(
            &user_sk,
            input_tx_hash,
            input_value,
            1_000_000,
            170_000,
            "preprod",
        );

        let wrong_node_hash = csl::PublicKey::from_bytes(wrong_node_sk.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_bytes();
        let user_hash = csl::PublicKey::from_bytes(user_sk.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_bytes();
        let datum_hex = build_datum_cbor_hex(user_hash, wrong_node_hash, vec![0u8; 32]);
        let provider_url = spawn_withdraw_provider_mock(
            "addr_test1script".to_string(),
            datum_hex,
            input_value,
            StatusCode::OK,
            request.tx_hash.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");
        seed_deposit(&ctx, mugraph_core::types::UtxoRef::new(input_tx_hash, 0), [0u8; 32]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("node_pubkey_hash mismatch"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request.tx_hash);
    }

    #[tokio::test]
    async fn handle_withdraw_surfaces_provider_input_verification_errors_without_mutating_state() {
        let user_sk = SigningKey::from_bytes(&[26u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let input_tx_hash = [0xc7u8; 32];
        let input_value = 1_170_000u64;
        let request = build_withdraw_request(
            &user_sk,
            input_tx_hash,
            input_value,
            1_000_000,
            170_000,
            "preprod",
        );

        let provider_url = spawn_withdraw_provider_mock_with_utxo_failure().await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");
        seed_deposit(&ctx, mugraph_core::types::UtxoRef::new(input_tx_hash, 0), [0u8; 32]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("Failed to verify input 0"));
        assert_preflight_rejection_leaves_state_untouched(&ctx, &request.tx_hash);
    }

    #[tokio::test]
    async fn handle_withdraw_submit_failure_marks_withdrawal_failed_without_unburning_notes() {
        let user_sk = SigningKey::from_bytes(&[4u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let input_tx_hash = [0xcdu8; 32];
        let input_value = 1_170_000u64;
        let request = build_withdraw_request(&user_sk, input_tx_hash, input_value, 1_000_000, 170_000, "preprod");

        let node_hash = csl::PublicKey::from_bytes(&payment_vk).unwrap().hash().to_bytes();
        let user_hash = csl::PublicKey::from_bytes(user_sk.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_bytes();
        let datum_hex = build_datum_cbor_hex(user_hash, node_hash, vec![0u8; 32]);
        let provider_url = spawn_withdraw_provider_mock(
            "addr_test1script".to_string(),
            datum_hex,
            input_value,
            StatusCode::INTERNAL_SERVER_ERROR,
            request.tx_hash.clone(),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");
        seed_deposit(&ctx, mugraph_core::types::UtxoRef::new(input_tx_hash, 0), [0u8; 32]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("Transaction submission failed"));

        let read_tx = ctx.database.read().unwrap();
        let notes = read_tx.open_table(NOTES).unwrap();
        let note_signature = mugraph_core::types::Signature::from([9u8; 32]);
        assert!(notes.get(note_signature).unwrap().is_some());

        let withdrawals = read_tx.open_table(WITHDRAWALS).unwrap();
        let key = withdrawal_key_from_hex(&request.tx_hash);
        assert_eq!(
            withdrawals.get(&key).unwrap().unwrap().value().status,
            mugraph_core::types::WithdrawalStatus::Failed
        );
    }

    #[tokio::test]
    async fn handle_withdraw_mismatched_submit_hash_marks_failed_without_spending_deposit() {
        let user_sk = SigningKey::from_bytes(&[20u8; 32]);
        let (payment_sk, payment_vk) = generate_payment_keypair().unwrap();
        let input_tx_hash = [0xc3u8; 32];
        let input_value = 1_170_000u64;
        let request = build_withdraw_request(
            &user_sk,
            input_tx_hash,
            input_value,
            1_000_000,
            170_000,
            "preprod",
        );

        let node_hash = csl::PublicKey::from_bytes(&payment_vk).unwrap().hash().to_bytes();
        let user_hash = csl::PublicKey::from_bytes(user_sk.verifying_key().as_bytes())
            .unwrap()
            .hash()
            .to_bytes();
        let datum_hex = build_datum_cbor_hex(user_hash, node_hash, vec![0u8; 32]);
        let provider_url = spawn_withdraw_provider_mock(
            "addr_test1script".to_string(),
            datum_hex,
            input_value,
            StatusCode::OK,
            "ee".repeat(32),
        )
        .await;
        let ctx = test_context_with_provider_url(Some(provider_url));
        insert_wallet(&ctx, payment_sk, payment_vk, "addr_test1script");
        let deposit_ref = mugraph_core::types::UtxoRef::new(input_tx_hash, 0);
        seed_deposit(&ctx, deposit_ref.clone(), [0u8; 32]);

        let err = handle_withdraw(&request, &ctx).await.unwrap_err();
        assert!(format!("{err:?}").contains("Provider returned mismatched tx hash"));

        let read_tx = ctx.database.read().unwrap();
        let notes = read_tx.open_table(NOTES).unwrap();
        let note_signature = mugraph_core::types::Signature::from([9u8; 32]);
        assert!(notes.get(note_signature).unwrap().is_some());

        let withdrawals = read_tx.open_table(WITHDRAWALS).unwrap();
        let key = withdrawal_key_from_hex(&request.tx_hash);
        assert_eq!(
            withdrawals.get(&key).unwrap().unwrap().value().status,
            mugraph_core::types::WithdrawalStatus::Failed
        );

        let deposits = read_tx.open_table(DEPOSITS).unwrap();
        assert!(!deposits.get(&deposit_ref).unwrap().unwrap().value().spent);
    }

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
            notes: vec![BlindSignature {
                signature: mugraph_core::types::Blinded(mugraph_core::types::Signature::from([1u8; 32])),
                proof: Default::default(),
            }],
        };

        atomic_burn_and_record_pending(&request, &ctx, &request.tx_hash)
            .expect("first burn succeeds");

        mark_withdrawal_failed(&ctx, &request.tx_hash)
            .expect("mark as failed");

        check_idempotency(&request, &ctx)
            .expect("failed withdrawals are retryable");

        let second = atomic_burn_and_record_pending(&request, &ctx, &request.tx_hash);
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
            notes: vec![BlindSignature {
                signature: mugraph_core::types::Blinded(mugraph_core::types::Signature::from([2u8; 32])),
                proof: Default::default(),
            }],
        };

        atomic_burn_and_record_pending(&request, &ctx, &request.tx_hash)
            .expect("first burn succeeds");

        let mismatched = "cd".repeat(32);
        let err = mark_withdrawal_completed(&ctx, &mismatched, &[]).unwrap_err();
        assert!(format!("{err:?}").contains("Pending withdrawal not found"));
    }

    #[test]
    fn completed_withdrawals_are_not_retryable() {
        let ctx = test_context();
        let request = WithdrawRequest {
            tx_hash: "ab".repeat(32),
            tx_cbor: "00".to_string(),
            notes: vec![],
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
    fn completion_rejects_already_completed_withdrawal_without_mutating_deposits() {
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

    #[test]
    fn test_intent_metadata_missing() {
        let tx = minimal_tx_with_values(1_000_000, 170_000);
        let err = validate_withdraw_intent_metadata(&tx.to_bytes(), "preprod").unwrap_err();
        assert!(format!("{:?}", err).contains("auxiliary data"));
    }

    #[test]
    fn test_intent_metadata_hash_mismatch() {
        let tx = tx_with_intent_metadata("preprod", Some("00".repeat(32)));
        let err = validate_withdraw_intent_metadata(&tx.to_bytes(), "preprod").unwrap_err();
        assert!(format!("{:?}", err).contains("tx_body_hash mismatch"));
    }

    #[test]
    fn test_intent_metadata_network_mismatch() {
        let tx = tx_with_intent_metadata("preprod", None);
        let err = validate_withdraw_intent_metadata(&tx.to_bytes(), "mainnet").unwrap_err();
        assert!(format!("{:?}", err).contains("network mismatch"));
    }

    #[test]
    fn test_multiasset_imbalance_rejected() {
        // Inputs: 1 ADA + 5 tokens; Outputs: 1 ADA + 6 tokens -> should fail
        let policy_hex = "00".repeat(28); // 28-byte script hash in hex
        let asset_hex = "746f6b656e"; // "token"
        let tx = tx_with_multiasset_output(1_000_000, &[(&policy_hex, asset_hex, 6)]);
        let tx_cbor = tx.to_bytes();
        let mut inputs = HashMap::new();
        inputs.insert("lovelace".to_string(), 1_000_000u128);
        inputs.insert(format!("{}{}", policy_hex, asset_hex), 5u128);
        let res = validate_transaction_balance(&tx_cbor, &inputs, 200_000);
        assert!(res.is_err());
    }

    #[test]
    fn test_multiasset_phantom_asset_rejected() {
        // Inputs: only ADA; Outputs: ADA + new token -> should fail
        let policy_hex = "00".repeat(28);
        let asset_hex = "746f6b656e";
        let tx = tx_with_multiasset_output(1_000_000, &[(&policy_hex, asset_hex, 1)]);
        let tx_cbor = tx.to_bytes();
        let mut inputs = HashMap::new();
        inputs.insert("lovelace".to_string(), 1_100_000u128); // cover fee + output
        let res = validate_transaction_balance(&tx_cbor, &inputs, 200_000);
        assert!(res.is_err());
    }

    fn tx_hash_from_body(body: &csl::TransactionBody) -> csl::TransactionHash {
        type Blake2b256 = blake2::Blake2b<blake2::digest::consts::U32>;
        let tx_hash = Blake2b256::digest(body.to_bytes());
        let mut tx_hash_arr = [0u8; 32];
        tx_hash_arr.copy_from_slice(&tx_hash);
        csl::TransactionHash::from_bytes(tx_hash_arr.to_vec()).unwrap()
    }

    fn witness_set_with_vkey_signers(
        tx_hash: &csl::TransactionHash,
        signers: &[&SigningKey],
    ) -> csl::TransactionWitnessSet {
        let mut witness_set = csl::TransactionWitnessSet::new();
        let mut vkeys = csl::Vkeywitnesses::new();

        for signer in signers {
            let private = csl::PrivateKey::from_normal_bytes(signer.as_bytes()).unwrap();
            let witness = csl::make_vkey_witness(tx_hash, &private);
            vkeys.add(&witness);
        }

        witness_set.set_vkeys(&vkeys);
        witness_set
    }

    fn minimal_tx_body_with_required_signers(
        signer_hash_hexes: &[String],
    ) -> csl::TransactionBody {
        let tx_hash = csl::TransactionHash::from_bytes(vec![0; 32]).unwrap();
        let input = csl::TransactionInput::new(&tx_hash, 0);
        let mut inputs = csl::TransactionInputs::new();
        inputs.add(&input);

        let addr = csl::Address::from_bech32(
            "addr_test1vru4e2un2tq50q4rv6qzk7t8w34gjdtw3y2uzuqxzj0ldrqqactxh",
        )
        .unwrap();
        let coin = csl::Coin::from_str("1000000").unwrap();
        let value = csl::Value::new(&coin);
        let output = csl::TransactionOutput::new(&addr, &value);
        let mut outputs = csl::TransactionOutputs::new();
        outputs.add(&output);

        let fee = csl::Coin::from_str("170000").unwrap();
        let mut body = csl::TransactionBody::new_tx_body(&inputs, &outputs, &fee);
        let mut required = csl::Ed25519KeyHashes::new();
        for signer_hash_hex in signer_hash_hexes {
            required.add(&csl::Ed25519KeyHash::from_hex(signer_hash_hex).unwrap());
        }
        body.set_required_signers(&required);
        body
    }

    fn minimal_tx_with_required_signer(
        signer_hash_hex: &str,
        witness_set: Option<csl::TransactionWitnessSet>,
    ) -> csl::Transaction {
        let body = minimal_tx_body_with_required_signers(&[signer_hash_hex.to_string()]);
        let witness_set = witness_set.unwrap_or_else(csl::TransactionWitnessSet::new);
        csl::Transaction::new(&body, &witness_set, None)
    }

    fn minimal_tx_with_values(output_lovelace: u64, fee: u64) -> csl::Transaction {
        let tx_hash = csl::TransactionHash::from_bytes(vec![0; 32]).unwrap();
        let input = csl::TransactionInput::new(&tx_hash, 0);
        let mut inputs = csl::TransactionInputs::new();
        inputs.add(&input);

        let addr = csl::Address::from_bech32(
            "addr_test1vru4e2un2tq50q4rv6qzk7t8w34gjdtw3y2uzuqxzj0ldrqqactxh",
        )
        .unwrap();
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

    fn tx_with_intent_metadata(network: &str, override_hash: Option<String>) -> csl::Transaction {
        let tx_hash = csl::TransactionHash::from_bytes(vec![0; 32]).unwrap();
        let input = csl::TransactionInput::new(&tx_hash, 0);
        let mut inputs = csl::TransactionInputs::new();
        inputs.add(&input);

        let addr = csl::Address::from_bech32(
            "addr_test1vru4e2un2tq50q4rv6qzk7t8w34gjdtw3y2uzuqxzj0ldrqqactxh",
        )
        .unwrap();
        let coin = csl::Coin::from_str("1000000").unwrap();
        let value = csl::Value::new(&coin);
        let output = csl::TransactionOutput::new(&addr, &value);
        let mut outputs = csl::TransactionOutputs::new();
        outputs.add(&output);

        let fee = csl::Coin::from_str("170000").unwrap();
        let body = csl::TransactionBody::new_tx_body(&inputs, &outputs, &fee);

        let body_hash_hex = if let Some(h) = override_hash {
            h
        } else {
            type Blake2b256 = blake2::Blake2b<blake2::digest::consts::U32>;
            let h = Blake2b256::digest(body.to_bytes());
            hex::encode(h)
        };

        let mut md_map = csl::MetadataMap::new();
        let md_network = csl::TransactionMetadatum::new_text(network.to_string()).unwrap();
        let md_hash = csl::TransactionMetadatum::new_text(body_hash_hex).unwrap();
        md_map.insert_str("network", &md_network).unwrap();
        md_map.insert_str("tx_body_hash", &md_hash).unwrap();
        let metadatum = csl::TransactionMetadatum::new_map(&md_map);
        let mut general_md = csl::GeneralTransactionMetadata::new();
        general_md.insert(&csl::BigNum::from_str("1914").unwrap(), &metadatum);
        let mut aux = csl::AuxiliaryData::new();
        aux.set_metadata(&general_md);

        let witness_set = csl::TransactionWitnessSet::new();
        csl::Transaction::new(&body, &witness_set, Some(aux))
    }

    fn tx_with_multiasset_output(
        lovelace: u64,
        assets: &[(&str, &str, u64)], // (policy_hex, asset_name_hex, qty)
    ) -> csl::Transaction {
        let tx_hash = csl::TransactionHash::from_bytes(vec![0; 32]).unwrap();
        let input = csl::TransactionInput::new(&tx_hash, 0);
        let mut inputs = csl::TransactionInputs::new();
        inputs.add(&input);

        let addr = csl::Address::from_bech32(
            "addr_test1vru4e2un2tq50q4rv6qzk7t8w34gjdtw3y2uzuqxzj0ldrqqactxh",
        )
        .unwrap();
        let coin = csl::Coin::from_str(&lovelace.to_string()).unwrap();
        let mut value = csl::Value::new(&coin);

        if !assets.is_empty() {
            let mut ma = csl::MultiAsset::new();
            for (policy_hex, asset_hex, qty) in assets {
                let policy = csl::ScriptHash::from_hex(policy_hex).unwrap();
                let mut assets_map = ma.get(&policy).unwrap_or_default();
                let name_bytes = hex::decode(asset_hex).unwrap();
                let name = csl::AssetName::new(name_bytes).unwrap();
                assets_map.insert(&name, &csl::BigNum::from_str(&qty.to_string()).unwrap());
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
