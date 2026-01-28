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
    let input_values =
        validate_script_inputs_with_deposits(&tx_bytes, &wallet.script_address, ctx, &provider)
            .await?;

    // 7. Validate user witnesses (basic count check)
    validate_user_witnesses(&tx_bytes, &request.notes, &wallet).await?;

    // 8. Validate transaction value balance
    let total_input: u64 = input_values.iter().map(|(_, amount)| amount).sum();
    validate_transaction_balance(&tx_bytes, total_input, ctx.config.max_withdrawal_fee())?;

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
            // CRITICAL: Submission failed but notes are already burned
            // We need to handle this failure case properly
            tracing::error!(
                "CRITICAL: Transaction submission failed after notes were burned: {}",
                e
            );

            // Mark withdrawal as failed in database for manual recovery
            if let Err(recovery_err) = mark_withdrawal_failed(ctx, &pending_tx_hash).await {
                tracing::error!(
                    "Failed to mark withdrawal as failed: {}. Manual recovery required for tx {}",
                    recovery_err,
                    pending_tx_hash
                );
            }

            return Err(Error::NetworkError {
                reason: format!(
                    "Transaction submission failed: {}. Notes have been burned but transaction was not submitted. Manual recovery required.",
                    e
                ),
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

/// Mark withdrawal as failed for recovery
async fn mark_withdrawal_failed(ctx: &Context, tx_hash: &str) -> Result<(), Error> {
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

        // Update record to mark as failed
        let record = WithdrawalRecord::failed();
        withdrawals_table.insert(key, &record)?;
    }

    write_tx.commit()?;

    tracing::warn!("Marked withdrawal {} as failed", tx_hash);

    Ok(())
}

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
            verified_witnesses += 1;
        }
    }

    if verified_witnesses == 0 {
        return Err(Error::InvalidSignature {
            reason: "No valid witnesses found in transaction".to_string(),
            signature: mugraph_core::types::Signature::default(),
        });
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
    script_address: &str,
    ctx: &Context,
    provider: &Provider,
) -> Result<Vec<(String, u64)>, Error> {
    use mugraph_core::types::UtxoRef;

    use crate::database::DEPOSITS;

    let inputs = extract_transaction_inputs(tx_cbor)?;

    if inputs.is_empty() {
        return Err(Error::InvalidInput {
            reason: "Transaction has no inputs".to_string(),
        });
    }

    let mut input_values: Vec<(String, u64)> = Vec::new();
    let read_tx = ctx.database.read()?;
    let deposits_table = read_tx.open_table(DEPOSITS)?;

    for (i, (tx_hash_bytes, index)) in inputs.iter().enumerate() {
        let tx_hash = hex::encode(tx_hash_bytes);

        tracing::debug!("Validating input {}: {}:{}", i, &tx_hash[..16], index);

        // Query blockchain to verify input is at script address
        match provider.get_utxo(&tx_hash, *index as u16).await {
            Ok(Some(utxo_info)) => {
                if utxo_info.address != script_address {
                    return Err(Error::InvalidInput {
                        reason: format!(
                            "Input {} ({}:{}) is not from script address. Expected {}, got {}",
                            i,
                            &tx_hash[..16],
                            index,
                            script_address,
                            utxo_info.address
                        ),
                    });
                }

                // Calculate total value of this UTxO
                let total_value: u64 = utxo_info
                    .amount
                    .iter()
                    .filter_map(|asset| asset.quantity.parse::<u64>().ok())
                    .sum();

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
                            "Input {}: deposit valid (block {}, expires {}), value: {} lovelace",
                            i,
                            deposit_record.block_height,
                            deposit_record.expires_at,
                            total_value
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

                input_values.push((tx_hash, total_value));
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
        "All {} inputs validated. Total input value: {} lovelace",
        inputs.len(),
        input_values.iter().map(|(_, v)| v).sum::<u64>()
    );

    Ok(input_values)
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
fn validate_transaction_balance(tx_cbor: &[u8], total_input: u64, max_fee: u64) -> Result<(), Error> {
    // Extract fee from transaction
    let fee = extract_transaction_fee(tx_cbor)?;

    // Extract outputs from transaction
    let outputs = extract_transaction_outputs(tx_cbor)?;
    let total_output: u64 = outputs.iter().map(|(_, amount)| amount).sum();

    tracing::info!(
        "Balance check: inputs={}, outputs={}, fee={}",
        total_input,
        total_output,
        fee
    );

    // Validate fee is within bounds
    if fee > max_fee {
        return Err(Error::InvalidInput {
            reason: format!("Fee {} lovelace exceeds maximum {} lovelace", fee, max_fee),
        });
    }

    // Validate conservation of value: inputs = outputs + fee
    let expected_input = total_output + fee;

    // Allow small tolerance for rounding/errors (0.1%)
    let tolerance = expected_input / 1000;
    let diff = total_input.abs_diff(expected_input);

    if diff > tolerance {
        return Err(Error::InvalidInput {
            reason: format!(
                "Transaction balance invalid. Inputs: {}, Expected (outputs + fee): {}, Difference: {} exceeds tolerance {}",
                total_input, expected_input, diff, tolerance
            ),
        });
    }

    tracing::info!(
        "Transaction balance valid. Inputs: {}, Outputs: {}, Fee: {}",
        total_input,
        total_output,
        fee
    );

    Ok(())
}
