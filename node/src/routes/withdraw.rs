use std::collections::{HashMap, HashSet};

use blake2::{Blake2b, Digest, digest::consts::U32};
use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{
        BlindSignature,
        Response,
        Signature,
        WithdrawRequest,
        WithdrawalKey,
        WithdrawalRecord,
        WithdrawalStatus,
    },
};
use redb::ReadableTable;
use whisky_csl::csl;

use crate::{
    database::{CARDANO_WALLET, NOTES, WITHDRAWALS},
    provider::Provider,
    routes::Context,
    tx_signer::{attach_witness_to_transaction, compute_tx_hash},
};

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
    let tx_bytes = hex::decode(&request.tx_cbor).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid tx_cbor hex: {}", e),
    })?;

    // Check transaction size limit (default: 16KB)
    let max_tx_size = 16384;
    if tx_bytes.len() > max_tx_size {
        return Err(Error::InvalidInput {
            reason: format!(
                "Transaction size {} exceeds maximum {}",
                tx_bytes.len(),
                max_tx_size
            ),
        });
    }

    // 2. Check idempotency via WITHDRAWALS table
    check_idempotency(request, ctx).await?;

    // 3. Validate transaction size and fee
    if tx_bytes.len() > ctx.config.max_tx_size() {
        return Err(Error::InvalidInput {
            reason: format!(
                "Transaction size {} bytes exceeds maximum {} bytes",
                tx_bytes.len(),
                ctx.config.max_tx_size()
            ),
        });
    }

    // Validate fee with tolerance
    let _fee = validate_fee(
        &tx_bytes,
        ctx.config.max_withdrawal_fee(),
        ctx.config.fee_tolerance_pct(),
    )?;

    // 4. Load wallet needed for validations and signing
    let wallet = load_wallet(ctx).await?;

    // 5. Verify provided hash matches recomputed hash
    let computed_hash = hex::encode(compute_tx_hash(&tx_bytes).map_err(|e| Error::InvalidInput {
        reason: format!("Failed to compute tx hash: {}", e),
    })?);
    if computed_hash != request.tx_hash {
        return Err(Error::InvalidInput {
            reason: format!(
                "Transaction hash mismatch: computed {}, provided {}",
                computed_hash, request.tx_hash
            ),
        });
    }

    // 6. Ensure all inputs reference script UTxOs and validate deposit state
    let (input_totals, required_user_hashes) =
        validate_script_inputs_with_deposits(&tx_bytes, &wallet, ctx, &provider).await?;

    // 7. Validate user witnesses (basic count check)
    validate_user_witnesses(&tx_bytes, &request.notes, &required_user_hashes, &wallet).await?;

    // 8. Validate transaction value balance
    validate_transaction_balance(&tx_bytes, &input_totals, ctx.config.max_withdrawal_fee())?;

    // 9. Create signed transaction (without burning notes yet)
    // This prepares the transaction for submission but doesn't modify state

    // Node signature is attached to the transaction witness set
    // The validator checks that the transaction is properly signed (off-chain verification)
    // No redeemer is needed - all validation happens through witnesses

    let tx_body_hash = crate::tx_signer::compute_tx_hash(&tx_bytes).map_err(|e| Error::Internal {
        reason: format!("Failed to compute tx hash: {}", e),
    })?;
    let signed_cbor =
        attach_witness_to_transaction(&tx_bytes, &tx_body_hash, &wallet).map_err(|e| {
            Error::Internal {
                reason: format!("Failed to sign transaction: {}", e),
            }
        })?;
    let signed_cbor_hex = hex::encode(&signed_cbor);

    // Calculate change notes before any state changes
    let change_notes = calculate_change_notes(request, &tx_bytes, &wallet)?;

    // 9. Update state atomically BEFORE submitting to provider
    // This ensures we only submit if we can properly track the withdrawal
    let pending_tx_hash = request.tx_hash.clone();
    match atomic_burn_and_record_pending(request, ctx, &pending_tx_hash).await {
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
            // Submission failed: rollback pending record and unburn notes
            tracing::error!(
                "Transaction submission failed after notes were burned: {}. Rolling back.",
                e
            );
            if let Err(rollback_err) =
                rollback_withdrawal(ctx, &pending_tx_hash, &request.notes).await
            {
                tracing::error!(
                    "Rollback failed after submission error: {}. Manual recovery needed for tx {}",
                    rollback_err,
                    pending_tx_hash
                );
            }

            return Err(Error::NetworkError {
                reason: format!("Transaction submission failed and was rolled back: {}", e),
            });
        }
    };

    // 11. Mark withdrawal as completed
    match mark_withdrawal_completed(ctx, &submit_response.tx_hash).await {
        Ok(()) => {
            tracing::info!(
                "Withdrawal completed successfully: {}",
                &submit_response.tx_hash[..std::cmp::min(16, submit_response.tx_hash.len())]
            );

            Ok(Response::Withdraw {
                signed_tx_cbor: signed_cbor_hex,
                tx_hash: submit_response.tx_hash,
                change_notes,
            })
        }
        Err(e) => {
            // CRITICAL: Transaction was submitted but we failed to mark as completed
            tracing::error!(
                "CRITICAL: Transaction {} was submitted but marking as completed failed: {}. Manual reconciliation required.",
                submit_response.tx_hash,
                e
            );

            // Still return success since blockchain transaction succeeded
            // but log the inconsistency for manual recovery
            Ok(Response::Withdraw {
                signed_tx_cbor: signed_cbor_hex,
                tx_hash: submit_response.tx_hash,
                change_notes,
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
async fn check_idempotency(request: &WithdrawRequest, ctx: &Context) -> Result<(), Error> {
    let read_tx = ctx.database.read()?;
    let table = read_tx.open_table(WITHDRAWALS)?;

    let tx_hash = hex::decode(&request.tx_hash).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid tx_hash hex: {}", e),
    })?;
    let tx_hash_array: [u8; 32] = tx_hash.try_into().map_err(|_| Error::InvalidInput {
        reason: "tx_hash must be 32 bytes".to_string(),
    })?;

    // Use network byte from config
    let network_byte = ctx.config.network_byte();
    let key = WithdrawalKey::new(network_byte, tx_hash_array);

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
async fn atomic_burn_and_record_pending(
    request: &WithdrawRequest,
    ctx: &Context,
    tx_hash: &str,
) -> Result<(), Error> {
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

        let tx_hash_bytes = hex::decode(tx_hash).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid tx_hash hex: {}", e),
        })?;
        let tx_hash_array: [u8; 32] = tx_hash_bytes.try_into().map_err(|_| Error::InvalidInput {
            reason: "tx_hash must be 32 bytes".to_string(),
        })?;

        let network_byte = ctx.config.network_byte();
        let key = WithdrawalKey::new(network_byte, tx_hash_array);

        // Create pending record
        let record = WithdrawalRecord::pending();
        withdrawals_table.insert(key, &record)?;
    }

    write_tx.commit()?;

    tracing::info!(
        "Burned {} notes and recorded pending withdrawal {}",
        request.notes.len(),
        &tx_hash[..std::cmp::min(16, tx_hash.len())]
    );

    Ok(())
}

/// Roll back pending withdrawal and unburn notes (best-effort)
async fn rollback_withdrawal(
    ctx: &Context,
    tx_hash: &str,
    notes: &[BlindSignature],
) -> Result<(), Error> {
    let write_tx = ctx.database.write()?;

    {
        // 1. Unburn notes
        let mut notes_table = write_tx.open_table(NOTES)?;
        for note in notes {
            let sig_bytes: &[u8; 32] = note.signature.0.as_ref();
            let signature = Signature::from(*sig_bytes);
            notes_table.remove(signature)?;
        }

        // 2. Remove pending withdrawal record
        let mut withdrawals_table = write_tx.open_table(WITHDRAWALS)?;
        let tx_hash_bytes = hex::decode(tx_hash).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid tx_hash hex: {}", e),
        })?;
        let tx_hash_array: [u8; 32] = tx_hash_bytes.try_into().map_err(|_| Error::InvalidInput {
            reason: "tx_hash must be 32 bytes".to_string(),
        })?;
        let network_byte = ctx.config.network_byte();
        let key = WithdrawalKey::new(network_byte, tx_hash_array);
        withdrawals_table.remove(key)?;
    }

    write_tx.commit()?;
    tracing::info!("Rolled back withdrawal {}", tx_hash);
    Ok(())
}

/// Mark withdrawal as failed for recovery
/// Mark withdrawal as completed after successful submission
async fn mark_withdrawal_completed(ctx: &Context, tx_hash: &str) -> Result<(), Error> {
    let write_tx = ctx.database.write()?;

    {
        let mut withdrawals_table = write_tx.open_table(WITHDRAWALS)?;

        let tx_hash_bytes = hex::decode(tx_hash).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid tx_hash hex: {}", e),
        })?;
        let tx_hash_array: [u8; 32] = tx_hash_bytes.try_into().map_err(|_| Error::InvalidInput {
            reason: "tx_hash must be 32 bytes".to_string(),
        })?;

        let network_byte = ctx.config.network_byte();
        let key = WithdrawalKey::new(network_byte, tx_hash_array);

        // Update record to mark as completed
        let record = WithdrawalRecord::completed();
        withdrawals_table.insert(key, &record)?;
    }

    write_tx.commit()?;

    tracing::info!("Marked withdrawal {} as completed", tx_hash);

    Ok(())
}

/// Load Cardano wallet for signing
async fn load_wallet(ctx: &Context) -> Result<mugraph_core::types::CardanoWallet, Error> {
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
fn validate_fee(tx_cbor: &[u8], max_fee_lovelace: u64, tolerance_pct: u8) -> Result<u64, Error> {
    let fee = extract_transaction_fee(tx_cbor)?;

    if fee > max_fee_lovelace {
        return Err(Error::InvalidInput {
            reason: format!(
                "Fee {} lovelace exceeds maximum {} lovelace",
                fee, max_fee_lovelace
            ),
        });
    }

    // Calculate acceptable fee range with tolerance
    // If tolerance is 5%, fee can be up to 105% of max_fee
    let tolerance_factor = 100 + tolerance_pct as u64;
    let max_acceptable_fee = max_fee_lovelace * tolerance_factor / 100;

    if fee > max_acceptable_fee {
        return Err(Error::InvalidInput {
            reason: format!(
                "Fee {} lovelace exceeds acceptable maximum {} lovelace (with {}% tolerance)",
                fee, max_acceptable_fee, tolerance_pct
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
fn extract_transaction_fee(tx_cbor: &[u8]) -> Result<u64, Error> {
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec()).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction CBOR: {}", e),
    })?;
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
fn extract_transaction_inputs(tx_cbor: &[u8]) -> Result<Vec<(Vec<u8>, u32)>, Error> {
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec()).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction CBOR: {}", e),
    })?;

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
async fn validate_user_witnesses(
    tx_cbor: &[u8],
    notes: &[mugraph_core::types::BlindSignature],
    expected_user_hashes: &HashSet<String>,
    _wallet: &mugraph_core::types::CardanoWallet,
) -> Result<(), Error> {
    // Parse full transaction using whisky-csl
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec()).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction CBOR: {}", e),
    })?;

    // Compute transaction body hash (BLAKE2b-256 over body bytes)
    let body_bytes = tx.body().to_bytes();
    let mut hasher = Blake2b::<U32>::new();
    hasher.update(&body_bytes);
    let body_hash = hasher.finalize();
    let body_hash_bytes: Vec<u8> = body_hash.to_vec();

    let witness_set = tx.witness_set();
    let mut verified_witnesses = 0usize;
    let mut witness_key_hashes: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Verify vkey witnesses
    if let Some(vkeys) = witness_set.vkeys() {
        for (idx, witness) in (&vkeys).into_iter().enumerate() {
            let pk: csl::PublicKey = witness.vkey().public_key();
            let sig = witness.signature();
            let ok = pk.verify(&body_hash_bytes, &sig);
            if !ok {
                return Err(Error::InvalidSignature {
                    reason: format!("VKey witness {} signature invalid", idx),
                    signature: mugraph_core::types::Signature::default(),
                });
            }
            witness_key_hashes.insert(pk.hash().to_hex());
            verified_witnesses += 1;
        }
    }

    // Verify bootstrap witnesses (Byron-era) if present
    if let Some(bootstraps) = witness_set.bootstraps() {
        for (idx, witness) in (&bootstraps).into_iter().enumerate() {
            let pk: csl::PublicKey = witness.vkey().public_key();
            let sig = witness.signature();
            let ok = pk.verify(&body_hash_bytes, &sig);
            if !ok {
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

    // Require witness set covers all note owners: we derive owner pubkey hash from notes' blinded sigs.
    // BlindSignature stores a blinded Signature; we can't invert it, so we encode owner expectation
    // via required_signers in the transaction. Enforce that required_signers are present.
    if let Some(required) = tx.body().required_signers() {
        let mut missing: Vec<String> = Vec::new();
        for idx in 0..required.len() {
            let h = required.get(idx);
            let hex = h.to_hex();
            if !witness_key_hashes.contains(&hex) {
                missing.push(hex);
            }
        }
        if !missing.is_empty() {
            return Err(Error::InvalidSignature {
                reason: format!("Missing witnesses for required_signers: {:?}", missing),
                signature: mugraph_core::types::Signature::default(),
            });
        }
    } else {
        return Err(Error::InvalidSignature {
            reason: "Transaction missing required_signers; cannot bind witnesses to note owners"
                .to_string(),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    // Check required_signers, if present, are covered by witnesses
    if let Some(required) = tx.body().required_signers() {
        for (idx, signer) in required.into_iter().enumerate() {
            let signer_hex = signer.to_hex();
            if !witness_key_hashes.contains(&signer_hex) {
                return Err(Error::InvalidSignature {
                    reason: format!(
                        "Missing witness for required_signer {} (index {})",
                        signer_hex, idx
                    ),
                    signature: mugraph_core::types::Signature::default(),
                });
            }
        }
    }

    // Bind witnesses to the owners (user_pubkey_hash) found in each input's datum
    // Every expected hash must appear in both required_signers and in the witness set.
    if expected_user_hashes.is_empty() {
        return Err(Error::InvalidSignature {
            reason: "No expected user hashes derived from inputs".to_string(),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    let required = tx
        .body()
        .required_signers()
        .ok_or_else(|| Error::InvalidSignature {
            reason: "Transaction missing required_signers; cannot bind witnesses to note owners"
                .to_string(),
            signature: mugraph_core::types::Signature::default(),
        })?;

    for expected in expected_user_hashes {
        let in_required = required.into_iter().any(|h| h.to_hex() == *expected);
        if !in_required {
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

    // Basic sanity: require at least as many witnesses as notes being burned
    if verified_witnesses < notes.len() {
        return Err(Error::InvalidSignature {
            reason: format!(
                "Not enough witnesses: found {}, but {} notes are being spent",
                verified_witnesses,
                notes.len()
            ),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    tracing::info!(
        "Validated {} witness signatures for {} notes",
        verified_witnesses,
        notes.len()
    );

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
    tx_cbor: &[u8],
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<Vec<BlindSignature>, Error> {
    // Extract outputs from transaction
    let outputs = extract_transaction_outputs(tx_cbor)?;

    // Identify change outputs (outputs to the user's address, not the script)
    // For now, we'll just log the outputs
    tracing::info!("Found {} transaction outputs", outputs.len());

    let mut change_amount: u64 = 0;

    for (i, (address, amount)) in outputs.iter().enumerate() {
        tracing::debug!("Output {}: address={}, amount={}", i, address, amount);

        // Check if this is a change output (not to script address)
        if address != &wallet.script_address {
            change_amount += amount;
            tracing::info!("Output {} is change: {} lovelace", i, amount);
        }
    }

    tracing::info!("Total change: {} lovelace", change_amount);

    // NOTE: Creating blind signatures for change notes requires:
    // 1. Blinding infrastructure for generating blind signatures (mugraph-core crypto)
    // 2. Mapping from output assets to note amounts
    // 3. Proper handling of multi-asset outputs
    //
    // This is a complex feature that requires integration with the note blinding system.
    // For now, we log the change amount but return no change notes.

    if change_amount > 0 {
        tracing::warn!(
            "Change of {} lovelace detected. Change notes will be implemented when blinding infrastructure is ready.",
            change_amount
        );
    }

    Ok(vec![])
}

/// Extract transaction outputs from CBOR
///
/// Returns a vector of (address, lovelace_amount) tuples
fn extract_transaction_outputs(tx_cbor: &[u8]) -> Result<Vec<(String, u64)>, Error> {
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec()).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction CBOR: {}", e),
    })?;

    let mut outputs: Vec<(String, u64)> = Vec::new();
    for output in &tx.body().outputs() {
        let address = output
            .address()
            .to_bech32(None)
            .map_err(|e| Error::InvalidInput {
                reason: format!("Invalid output address: {}", e),
            })?;
        let coin = output.amount().coin();
        let amount = coin
            .to_str()
            .parse::<u64>()
            .map_err(|e| Error::InvalidInput {
                reason: format!("Invalid output amount: {}", e),
            })?;
        outputs.push((address, amount));
    }

    Ok(outputs)
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
async fn validate_script_inputs_with_deposits(
    tx_cbor: &[u8],
    wallet: &mugraph_core::types::CardanoWallet,
    ctx: &Context,
    provider: &Provider,
) -> Result<(HashMap<String, u128>, HashSet<String>), Error> {
    use mugraph_core::types::UtxoRef;

    use crate::database::DEPOSITS;

    let inputs = extract_transaction_inputs(tx_cbor)?;

    if inputs.is_empty() {
        return Err(Error::InvalidInput {
            reason: "Transaction has no inputs".to_string(),
        });
    }

    let mut totals: HashMap<String, u128> = HashMap::new();
    let mut required_user_hashes: HashSet<String> = HashSet::new();
    let read_tx = ctx.database.read()?;
    let deposits_table = read_tx.open_table(DEPOSITS)?;

    // Pre-compute node pubkey hash (blake2b-224) to compare with datum
    let node_pk = csl::PublicKey::from_bytes(&wallet.payment_vk).map_err(|e| Error::InvalidKey {
        reason: format!("Invalid node payment_vk: {}", e),
    })?;
    let node_pk_hash = node_pk.hash().to_bytes();

    for (i, (tx_hash_bytes, index)) in inputs.iter().enumerate() {
        let tx_hash = hex::encode(tx_hash_bytes);

        tracing::debug!("Validating input {}: {}:{}", i, &tx_hash[..16], index);

        // Query blockchain to verify input is at script address
        match provider.get_utxo(&tx_hash, *index as u16).await {
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

                let datum_bytes = hex::decode(datum_hex).map_err(|e| Error::InvalidInput {
                    reason: format!("Invalid datum hex for input {}: {}", i, e),
                })?;

                let pd =
                    csl::PlutusData::from_bytes(datum_bytes).map_err(|e| Error::InvalidInput {
                        reason: format!("Invalid datum CBOR for input {}: {}", i, e),
                    })?;

                let constr = pd
                    .as_constr_plutus_data()
                    .ok_or_else(|| Error::InvalidInput {
                        reason: format!("Datum for input {} is not a constructor as expected", i),
                    })?;

                let alt = constr.alternative().to_str();
                if alt != "0" {
                    return Err(Error::InvalidInput {
                        reason: format!(
                            "Unexpected datum constructor {} for input {} (expected 0)",
                            alt, i
                        ),
                    });
                }

                let fields = constr.data();
                if fields.len() != 3 {
                    return Err(Error::InvalidInput {
                        reason: format!(
                            "Datum for input {} has {} fields (expected 3)",
                            i,
                            fields.len()
                        ),
                    });
                }

                // Field 0: user_pubkey_hash
                let user_hash = fields
                    .get(0)
                    .as_bytes()
                    .ok_or_else(|| Error::InvalidInput {
                        reason: format!("Datum for input {} missing user_pubkey_hash bytes", i),
                    })?;

                required_user_hashes.insert(hex::encode(user_hash));

                // Field 1: node_pubkey_hash
                let node_hash = fields
                    .get(1)
                    .as_bytes()
                    .ok_or_else(|| Error::InvalidInput {
                        reason: format!("Datum for input {} missing node_pubkey_hash bytes", i),
                    })?;

                if node_hash != node_pk_hash {
                    return Err(Error::InvalidInput {
                        reason: format!(
                            "Input {} node_pubkey_hash mismatch; expected our node, got {}",
                            i,
                            hex::encode(node_hash)
                        ),
                    });
                }

                // Field 2: intent_hash
                let intent_hash = fields
                    .get(2)
                    .as_bytes()
                    .ok_or_else(|| Error::InvalidInput {
                        reason: format!("Datum for input {} missing intent_hash bytes", i),
                    })?;

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
                let utxo_ref = UtxoRef::new(tx_hash_array, *index as u16);

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
                            && intent_hash.as_slice() != deposit_record.intent_hash
                        {
                            return Err(Error::InvalidInput {
                                reason: format!(
                                    "Intent hash mismatch for input {}: datum {}, expected {}",
                                    i,
                                    hex::encode(intent_hash),
                                    hex::encode(deposit_record.intent_hash)
                                ),
                            });
                        }

                        // Check if expired
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
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

    Ok((totals, required_user_hashes))
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
fn validate_transaction_balance(
    tx_cbor: &[u8],
    input_totals: &HashMap<String, u128>,
    max_fee: u64,
) -> Result<(), Error> {
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec()).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction CBOR: {}", e),
    })?;

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

    // Apply fee to lovelace output total
    let out_lovelace = output_totals.entry("lovelace".to_string()).or_insert(0);
    *out_lovelace = out_lovelace.saturating_add(fee_u128);

    // Compare per-asset
    for (unit, in_qty) in input_totals.iter() {
        let out_qty = output_totals.get(unit).copied().unwrap_or(0);
        if unit == "lovelace" {
            // Allow 0.1% tolerance on lovelace
            let expected = out_qty;
            let tolerance = expected / 1000;
            let diff = (*in_qty).abs_diff(expected);
            if diff > tolerance {
                return Err(Error::InvalidInput {
                    reason: format!(
                        "Lovelace imbalance: inputs {}, outputs+fee {}, diff {}, tolerance {}",
                        in_qty, expected, diff, tolerance
                    ),
                });
            }
        } else if *in_qty != out_qty {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Asset imbalance for {}: inputs {}, outputs {}",
                    unit, in_qty, out_qty
                ),
            });
        }
    }

    // Also ensure no extra assets are minted/appearing
    for (unit, out_qty) in output_totals.iter() {
        let in_qty = input_totals.get(unit).copied().unwrap_or(0);
        if unit == "lovelace" {
            continue; // already checked with tolerance
        }
        if *out_qty > in_qty {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Outputs create extra asset {}: outputs {}, inputs {}",
                    unit, out_qty, in_qty
                ),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;

    use super::*;

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
        let body_bytes = tx_body_only.to_bytes();

        // Sign body hash using CSL helper
        type Blake2b256 = blake2::Blake2b<blake2::digest::consts::U32>;
        let tx_hash = Blake2b256::digest(&body_bytes);
        let mut tx_hash_arr = [0u8; 32];
        tx_hash_arr.copy_from_slice(&tx_hash);
        let tx_hash_csl = csl::TransactionHash::from_bytes(tx_hash_arr.to_vec()).unwrap();
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

    fn minimal_tx_with_required_signer(
        signer_hash_hex: &str,
        witness_set: Option<csl::TransactionWitnessSet>,
    ) -> csl::Transaction {
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
        let signer_hash = csl::Ed25519KeyHash::from_hex(signer_hash_hex).unwrap();
        let mut required = csl::Ed25519KeyHashes::new();
        required.add(&signer_hash);
        body.set_required_signers(&required);

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
}
