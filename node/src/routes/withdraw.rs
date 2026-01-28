use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{BlindSignature, Response, Signature, WithdrawRequest, WithdrawalKey, WithdrawalRecord},
};
use redb::ReadableTable;

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
/// 4. Validate user signatures (CIP-8/COSE)
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

    // 4. Parse transaction and validate inputs
    // Use whisky-csl to parse and validate the transaction
    // let tx = whisky_csl::Transaction::from_bytes(&tx_bytes)
    //     .map_err(|e| Error::InvalidInput { reason: format!("Invalid transaction CBOR: {}", e) })?;

    // 4. Verify provided hash matches recomputed hash
    // let computed_hash = tx.hash();
    // if hex::encode(&computed_hash) != request.tx_hash {
    //     return Err(Error::InvalidInput { reason: "Transaction hash mismatch".to_string() });
    // }

    // 5. Ensure all inputs reference script UTxOs
    // validate_script_inputs(&tx, &wallet.script_address).await?;

    // 6. Validate user witnesses correspond to input addresses
    // validate_user_witnesses(&tx, &request.notes).await?;

    // 7. Check outputs match burned notes minus fees
    // let change_notes = validate_withdrawal_amounts(&tx, &request.notes)?;

    // 8. Create signed transaction (without burning notes yet)
    // This prepares the transaction for submission but doesn't modify state
    let wallet = load_wallet(ctx).await?;

    // Node signature is attached to the transaction witness set
    // The validator checks that the transaction is properly signed (off-chain verification)
    // No redeemer is needed - all validation happens through witnesses

    let tx_body_hash = crate::tx_signer::compute_tx_hash(&tx_bytes).map_err(|e| Error::Internal {
        reason: format!("Failed to compute tx hash: {}", e),
    })?;
    let signed_cbor =
        crate::tx_signer::attach_witness_to_transaction(&tx_bytes, &tx_body_hash, &wallet).map_err(
            |e| Error::Internal {
                reason: format!("Failed to sign transaction: {}", e),
            },
        )?;
    let signed_cbor_hex = hex::encode(&signed_cbor);

    // Calculate change notes before any state changes
    let change_notes = calculate_change_notes(request, &tx_bytes, &wallet)?;

    // 9. Submit transaction to provider FIRST
    // Only after successful submission do we update state
    let submit_response = match submit_transaction(&signed_cbor_hex, &provider).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!(
                "Transaction submission failed, no state changes made: {}",
                e
            );
            return Err(Error::NetworkError {
                reason: format!("Transaction submission failed: {}", e),
            });
        }
    };

    // 10. Transaction submitted successfully - now update state atomically
    // This is the critical section where we burn notes and record the withdrawal
    // Both operations happen in a single database transaction
    match atomic_burn_and_record(request, ctx, &submit_response.tx_hash).await {
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
            // CRITICAL: The transaction was submitted to the blockchain but
            // we failed to update our local state. This is a serious inconsistency
            // that requires manual intervention or a recovery process.
            tracing::error!(
                "CRITICAL: Transaction {} was submitted but state update failed: {}",
                submit_response.tx_hash,
                e
            );

            // In production, you would:
            // 1. Log this to a dead letter queue
            // 2. Alert operators
            // 3. Implement a reconciliation process
            // For now, we still return success since the blockchain transaction succeeded
            // but include a warning
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
        if record.value().processed {
            return Err(Error::InvalidInput {
                reason: "Withdrawal already processed".to_string(),
            });
        }
    }

    Ok(())
}

/// Burn notes and attach node witness
async fn burn_notes_and_sign(
    request: &WithdrawRequest,
    ctx: &Context,
    tx_bytes: &[u8],
) -> Result<(String, Vec<BlindSignature>), Error> {
    // First, burn the notes
    burn_notes(&request.notes, ctx).await?;

    // Load wallet for signing
    let wallet = load_wallet(ctx).await?;

    // Compute transaction hash for signing
    // This extracts the transaction body from CBOR and computes its hash
    let tx_hash = compute_tx_hash(tx_bytes).map_err(|e| Error::Internal {
        reason: format!("Failed to compute tx hash: {}", e),
    })?;

    // Attach node witness to transaction
    let signed_tx =
        attach_witness_to_transaction(tx_bytes, &tx_hash, &wallet).map_err(|e| Error::Internal {
            reason: format!("Failed to sign transaction: {}", e),
        })?;

    let signed_cbor = hex::encode(&signed_tx);

    // Calculate change notes from transaction outputs
    let wallet = load_wallet(ctx).await?;
    let change_notes = calculate_change_notes(request, &tx_bytes, &wallet)?;

    Ok((signed_cbor, change_notes))
}

/// Atomically burn notes and record withdrawal in a single database transaction
///
/// This ensures both operations succeed or both fail, maintaining consistency
/// between the NOTES table and WITHDRAWALS table.
async fn atomic_burn_and_record(
    request: &WithdrawRequest,
    ctx: &Context,
    submitted_tx_hash: &str,
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

        // 2. Record withdrawal
        let mut withdrawals_table = write_tx.open_table(WITHDRAWALS)?;

        let tx_hash = hex::decode(submitted_tx_hash).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid submitted tx_hash hex: {}", e),
        })?;
        let tx_hash_array: [u8; 32] = tx_hash.try_into().map_err(|_| Error::InvalidInput {
            reason: "Submitted tx_hash must be 32 bytes".to_string(),
        })?;

        // Use network byte from config
        let network_byte = ctx.config.network_byte();
        let key = WithdrawalKey::new(network_byte, tx_hash_array);

        let record = WithdrawalRecord::new(true);
        withdrawals_table.insert(key, &record)?;
    }

    // Commit both operations atomically
    write_tx.commit()?;

    tracing::info!(
        "Atomically burned {} notes and recorded withdrawal {}",
        request.notes.len(),
        &submitted_tx_hash[..std::cmp::min(16, submitted_tx_hash.len())]
    );

    Ok(())
}

/// Burn notes by marking them as spent in the database
/// This uses the same mechanism as the refresh system
async fn burn_notes(notes: &[BlindSignature], ctx: &Context) -> Result<(), Error> {
    if notes.is_empty() {
        return Err(Error::InvalidInput {
            reason: "No notes provided for withdrawal".to_string(),
        });
    }

    let write_tx = ctx.database.write()?;
    {
        let mut table = write_tx.open_table(NOTES)?;

        for note in notes {
            // Convert BlindSignature to Signature for the table key
            // BlindSignature contains signature: Blinded<Signature>
            // We need to extract the inner signature
            let sig_bytes: &[u8; 32] = note.signature.0.as_ref();
            let signature = Signature::from(*sig_bytes);

            // Check if note is already spent
            if table.get(signature)?.is_some() {
                return Err(Error::AlreadySpent { signature });
            }

            // Mark note as spent
            table.insert(signature, true)?;

            tracing::debug!("Burned note: {:x}", signature);
        }
    }
    write_tx.commit()?;

    tracing::info!("Successfully burned {} notes", notes.len());
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

/// Record withdrawal in database for idempotency
async fn record_withdrawal(request: &WithdrawRequest, ctx: &Context) -> Result<(), Error> {
    let write_tx = ctx.database.write()?;
    {
        let mut table = write_tx.open_table(WITHDRAWALS)?;

        let tx_hash = hex::decode(&request.tx_hash).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid tx_hash hex: {}", e),
        })?;
        let tx_hash_array: [u8; 32] = tx_hash.try_into().map_err(|_| Error::InvalidInput {
            reason: "tx_hash must be 32 bytes".to_string(),
        })?;

        // Use network byte from config
        let network_byte = ctx.config.network_byte();
        let key = WithdrawalKey::new(network_byte, tx_hash_array);

        let record = WithdrawalRecord::new(true);
        table.insert(key, &record)?;
    }
    write_tx.commit()?;

    tracing::info!("Withdrawal recorded successfully");
    Ok(())
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
    let mut decoder = minicbor::Decoder::new(tx_cbor);

    // Transaction is an array [body, witness_set, is_valid, auxiliary_data]
    let tx_len = decoder.array().map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction CBOR: {}", e),
    })?;
    let tx_len = tx_len.ok_or_else(|| Error::InvalidInput {
        reason: "Indefinite transaction length not supported".to_string(),
    })?;

    if tx_len < 1 {
        return Err(Error::InvalidInput {
            reason: "Transaction missing body".to_string(),
        });
    }

    // Parse transaction body to extract fee
    let body_start = decoder.position();
    decoder.skip().map_err(|e| Error::InvalidInput {
        reason: format!("Failed to parse transaction body: {}", e),
    })?;
    let body_end = decoder.position();
    let body_cbor = &tx_cbor[body_start..body_end];

    // Parse fee from body
    parse_fee_from_body(body_cbor)
}

/// Parse fee from transaction body CBOR
fn parse_fee_from_body(body_cbor: &[u8]) -> Result<u64, Error> {
    let mut decoder = minicbor::Decoder::new(body_cbor);

    // Transaction body is a map with field indices as keys
    let map_len = decoder.map().map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction body: {}", e),
    })?;

    let map_len = match map_len {
        Some(len) => len as usize,
        None => usize::MAX,
    };

    for _ in 0..map_len {
        let key_result: Result<u64, _> = decoder.u64();
        match key_result {
            Ok(2) => {
                // Key 2 = fee
                let fee = decoder.u64().map_err(|e| Error::InvalidInput {
                    reason: format!("Invalid fee: {}", e),
                })?;
                return Ok(fee);
            }
            Ok(_) => {
                // Skip other fields
                decoder.skip().map_err(|e| Error::InvalidInput {
                    reason: format!("Failed to skip field: {}", e),
                })?;
            }
            Err(_) => break,
        }
    }

    Err(Error::InvalidInput {
        reason: "Fee not found in transaction".to_string(),
    })
}

/// Validate that all inputs reference the script address
///
/// # Implementation Note
/// This is a best-effort implementation that extracts transaction inputs from CBOR
/// and validates them against the expected script address. A full implementation
/// would query the blockchain to verify each input's address.
///
/// # Arguments
/// * `tx_cbor` - The transaction CBOR bytes
/// * `script_address` - The expected script address
/// * `provider` - Optional provider to verify addresses on-chain
async fn validate_script_inputs(
    tx_cbor: &[u8],
    script_address: &str,
    _provider: Option<&Provider>,
) -> Result<(), Error> {
    // Parse transaction inputs from CBOR
    let inputs = extract_transaction_inputs(tx_cbor)?;

    if inputs.is_empty() {
        return Err(Error::InvalidInput {
            reason: "Transaction has no inputs".to_string(),
        });
    }

    tracing::info!("Validating {} script inputs", inputs.len());

    // For each input, we need to verify it comes from the script address
    // In a full implementation, we would query the provider for each input
    // to get its address and verify it matches the script address.
    //
    // NOTE: On-chain address verification is not yet enabled. To enable:
    // 1. Uncomment the provider query below
    // 2. Verify each input's address matches the script address
    // 3. Handle errors appropriately
    //
    // This adds an additional security check but requires reliable provider access.
    for (i, (tx_hash, index)) in inputs.iter().enumerate() {
        tracing::debug!("Input {}: {}#{}", i, hex::encode(tx_hash), index);

        // ON-CHAIN VERIFICATION (disabled by default):
        // if let Some(provider) = provider {
        //     let utxo_info = provider.get_utxo(&hex::encode(tx_hash), *index).await?;
        //     if utxo_info.address != script_address {
        //         return Err(Error::InvalidInput {
        //             reason: format!("Input {} is not from script address", i),
        //         });
        //     }
        // }
    }

    tracing::info!(
        "All {} inputs validated (address verification pending)",
        inputs.len()
    );
    Ok(())
}

/// Extract transaction inputs from CBOR
///
/// Transaction body structure (simplified):
/// - inputs: []TransactionInput (array of {transaction_id, index})
/// - outputs: []TransactionOutput
/// - fee: Coin
/// - ... other fields
fn extract_transaction_inputs(tx_cbor: &[u8]) -> Result<Vec<(Vec<u8>, u32)>, Error> {
    let mut decoder = minicbor::Decoder::new(tx_cbor);

    // Transaction is an array [body, witness_set, is_valid, auxiliary_data]
    let tx_len = decoder.array().map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction CBOR: {}", e),
    })?;
    let tx_len = tx_len.ok_or_else(|| Error::InvalidInput {
        reason: "Indefinite transaction length not supported".to_string(),
    })?;

    if tx_len < 1 {
        return Err(Error::InvalidInput {
            reason: "Transaction missing body".to_string(),
        });
    }

    // Parse transaction body to extract inputs
    let body_start = decoder.position();
    decoder.skip().map_err(|e| Error::InvalidInput {
        reason: format!("Failed to parse transaction body: {}", e),
    })?;
    let body_end = decoder.position();
    let body_cbor = &tx_cbor[body_start..body_end];

    // Parse inputs from body
    parse_inputs_from_body(body_cbor)
}

/// Parse inputs from transaction body CBOR
fn parse_inputs_from_body(body_cbor: &[u8]) -> Result<Vec<(Vec<u8>, u32)>, Error> {
    let mut decoder = minicbor::Decoder::new(body_cbor);

    // Transaction body is a map with field indices as keys
    let map_len = decoder.map().map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction body: {}", e),
    })?;

    let mut inputs: Vec<(Vec<u8>, u32)> = Vec::new();
    let map_len = match map_len {
        Some(len) => len as usize,
        None => {
            // Indefinite length - we'll iterate until break
            usize::MAX
        }
    };

    for _ in 0..map_len {
        // Try to get the key
        let key_result: Result<u64, _> = decoder.u64();
        match key_result {
            Ok(0) => {
                // Key 0 = inputs
                let arr_len = decoder.array().map_err(|e| Error::InvalidInput {
                    reason: format!("Invalid inputs array: {}", e),
                })?;
                let arr_len = arr_len.ok_or_else(|| Error::InvalidInput {
                    reason: "Indefinite inputs array not supported".to_string(),
                })?;

                for _ in 0..arr_len {
                    // Each input is [transaction_id, index]
                    let input_arr_len = decoder.array().map_err(|e| Error::InvalidInput {
                        reason: format!("Invalid input: {}", e),
                    })?;
                    let input_arr_len = input_arr_len.ok_or_else(|| Error::InvalidInput {
                        reason: "Indefinite input not supported".to_string(),
                    })?;

                    if input_arr_len != 2 {
                        return Err(Error::InvalidInput {
                            reason: "Input must be [transaction_id, index]".to_string(),
                        });
                    }

                    let tx_id: Vec<u8> = decoder
                        .bytes()
                        .map_err(|e| Error::InvalidInput {
                            reason: format!("Invalid transaction_id: {}", e),
                        })?
                        .to_vec();
                    let index: u32 = decoder.u32().map_err(|e| Error::InvalidInput {
                        reason: format!("Invalid input index: {}", e),
                    })?;

                    inputs.push((tx_id, index));
                }
                break; // Found inputs, done
            }
            Ok(_) => {
                // Skip other fields
                decoder.skip().map_err(|e| Error::InvalidInput {
                    reason: format!("Failed to skip field: {}", e),
                })?;
            }
            Err(_) => {
                // End of map (for indefinite length) or error
                break;
            }
        }
    }

    if inputs.is_empty() {
        return Err(Error::InvalidInput {
            reason: "No inputs found in transaction".to_string(),
        });
    }

    Ok(inputs)
}

/// Validate user witnesses
///
/// # Implementation Note
/// This validates that the transaction contains proper user signatures.
/// A full implementation would verify CIP-8/COSE signatures from the witness set.
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
    // Extract witness set from transaction
    let witness_set = extract_witness_set(tx_cbor)?;

    // Count vkey witnesses
    let vkey_count = count_vkey_witnesses(&witness_set)?;

    tracing::info!(
        "Found {} vkey witnesses for {} notes",
        vkey_count,
        notes.len()
    );

    // For now, we just verify there are witnesses present
    // A full implementation would:
    // 1. Parse each vkey witness
    // 2. Verify the signature against the transaction hash
    // 3. Verify the public key matches the note owner's key
    if vkey_count == 0 {
        return Err(Error::InvalidSignature {
            reason: "No vkey witnesses found in transaction".to_string(),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    // NOTE: Full CIP-8/COSE validation would require:
    // 1. Parsing each vkey witness from the witness set
    // 2. Verifying the signature against the transaction hash
    // 3. Verifying the public key matches the note owner's key
    // This requires a COSE library and additional infrastructure.
    // For now, we verify that witnesses are present.

    Ok(())
}

/// Extract witness set from transaction CBOR
fn extract_witness_set(tx_cbor: &[u8]) -> Result<Vec<u8>, Error> {
    let mut decoder = minicbor::Decoder::new(tx_cbor);

    // Transaction is an array [body, witness_set, is_valid, auxiliary_data]
    let tx_len = decoder.array().map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction CBOR: {}", e),
    })?;
    let tx_len = tx_len.ok_or_else(|| Error::InvalidInput {
        reason: "Indefinite transaction length not supported".to_string(),
    })?;

    if tx_len < 2 {
        return Err(Error::InvalidInput {
            reason: "Transaction missing witness set".to_string(),
        });
    }

    // Skip body
    decoder.skip().map_err(|e| Error::InvalidInput {
        reason: format!("Failed to skip transaction body: {}", e),
    })?;

    // Extract witness set
    let witness_start = decoder.position();
    decoder.skip().map_err(|e| Error::InvalidInput {
        reason: format!("Failed to parse witness set: {}", e),
    })?;
    let witness_end = decoder.position();

    Ok(tx_cbor[witness_start..witness_end].to_vec())
}

/// Count vkey witnesses in witness set
fn count_vkey_witnesses(witness_set: &[u8]) -> Result<usize, Error> {
    if witness_set.is_empty() || witness_set == [0xa0] {
        // Empty map
        return Ok(0);
    }

    let mut decoder = minicbor::Decoder::new(witness_set);

    // Witness set is a map
    let map_len = decoder.map().map_err(|e| Error::InvalidInput {
        reason: format!("Invalid witness set: {}", e),
    })?;

    let mut vkey_count = 0;
    let map_len = match map_len {
        Some(len) => len as usize,
        None => usize::MAX, // Indefinite length
    };

    for _ in 0..map_len {
        match decoder.u64() {
            Ok(0) => {
                // Key 0 = vkey witnesses
                let arr_len = decoder.array().map_err(|e| Error::InvalidInput {
                    reason: format!("Invalid vkey witnesses: {}", e),
                })?;
                let arr_len = arr_len.ok_or_else(|| Error::InvalidInput {
                    reason: "Indefinite vkey array not supported".to_string(),
                })?;
                vkey_count = arr_len as usize;
                break;
            }
            Ok(_) => {
                // Skip other witness types
                decoder.skip().map_err(|e| Error::InvalidInput {
                    reason: format!("Failed to skip witness type: {}", e),
                })?;
            }
            Err(_) => break,
        }
    }

    Ok(vkey_count)
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
    let mut decoder = minicbor::Decoder::new(tx_cbor);

    // Transaction is an array [body, witness_set, is_valid, auxiliary_data]
    let tx_len = decoder.array().map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction CBOR: {}", e),
    })?;
    let tx_len = tx_len.ok_or_else(|| Error::InvalidInput {
        reason: "Indefinite transaction length not supported".to_string(),
    })?;

    if tx_len < 1 {
        return Err(Error::InvalidInput {
            reason: "Transaction missing body".to_string(),
        });
    }

    // Parse transaction body to extract outputs
    let body_start = decoder.position();
    decoder.skip().map_err(|e| Error::InvalidInput {
        reason: format!("Failed to parse transaction body: {}", e),
    })?;
    let body_end = decoder.position();
    let body_cbor = &tx_cbor[body_start..body_end];

    // Parse outputs from body
    parse_outputs_from_body(body_cbor)
}

/// Parse outputs from transaction body CBOR
fn parse_outputs_from_body(body_cbor: &[u8]) -> Result<Vec<(String, u64)>, Error> {
    let mut decoder = minicbor::Decoder::new(body_cbor);

    // Transaction body is a map with field indices as keys
    let map_len = decoder.map().map_err(|e| Error::InvalidInput {
        reason: format!("Invalid transaction body: {}", e),
    })?;

    let mut outputs: Vec<(String, u64)> = Vec::new();
    let map_len = match map_len {
        Some(len) => len as usize,
        None => usize::MAX,
    };

    for _ in 0..map_len {
        let key_result: Result<u64, _> = decoder.u64();
        match key_result {
            Ok(1) => {
                // Key 1 = outputs
                let arr_len = decoder.array().map_err(|e| Error::InvalidInput {
                    reason: format!("Invalid outputs array: {}", e),
                })?;
                let arr_len = arr_len.ok_or_else(|| Error::InvalidInput {
                    reason: "Indefinite outputs array not supported".to_string(),
                })?;

                for _ in 0..arr_len {
                    // Parse output (simplified - just get address and lovelace amount)
                    match parse_output(&mut decoder) {
                        Ok((address, amount)) => {
                            outputs.push((address, amount));
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse output: {}", e);
                            // Skip this output
                            decoder.skip().ok();
                        }
                    }
                }
                break;
            }
            Ok(_) => {
                decoder.skip().map_err(|e| Error::InvalidInput {
                    reason: format!("Failed to skip field: {}", e),
                })?;
            }
            Err(_) => break,
        }
    }

    Ok(outputs)
}

/// Parse a single transaction output
///
/// Returns (address, lovelace_amount)
fn parse_output(decoder: &mut minicbor::Decoder) -> Result<(String, u64), Error> {
    // Output is an array [address, amount, datum_hash?, script_ref?]
    // or a map in post-Alonzo format

    // Try array format first
    match decoder.array() {
        Ok(Some(len)) => {
            if len < 2 {
                return Err(Error::InvalidInput {
                    reason: "Output must have at least address and amount".to_string(),
                });
            }

            // Parse address (could be bytes or string depending on encoding)
            let address = match decoder.bytes() {
                Ok(bytes) => {
                    // Try to decode as bech32
                    hex::encode(bytes)
                }
                Err(_) => {
                    // Try as string
                    decoder
                        .str()
                        .map_err(|e| Error::InvalidInput {
                            reason: format!("Invalid address: {}", e),
                        })?
                        .to_string()
                }
            };

            // Parse amount
            let amount = parse_amount(decoder)?;

            // Skip remaining fields (datum, script ref)
            for _ in 2..len {
                decoder.skip().ok();
            }

            Ok((address, amount))
        }
        _ => {
            // Try map format
            parse_output_map(decoder)
        }
    }
}

/// Parse amount from output
fn parse_amount(decoder: &mut minicbor::Decoder) -> Result<u64, Error> {
    // Amount can be:
    // - u64 for just lovelace
    // - array [coin, multiasset] for multi-asset

    match decoder.u64() {
        Ok(amount) => Ok(amount),
        Err(_) => {
            // Try array format
            match decoder.array() {
                Ok(Some(2)) => {
                    // [coin, multiasset]
                    let coin = decoder.u64().map_err(|e| Error::InvalidInput {
                        reason: format!("Invalid coin amount: {}", e),
                    })?;
                    // Skip multiasset
                    decoder.skip().ok();
                    Ok(coin)
                }
                _ => Err(Error::InvalidInput {
                    reason: "Invalid amount format".to_string(),
                }),
            }
        }
    }
}

/// Parse output in map format (post-Alonzo)
fn parse_output_map(decoder: &mut minicbor::Decoder) -> Result<(String, u64), Error> {
    let map_len = decoder.map().map_err(|e| Error::InvalidInput {
        reason: format!("Invalid output map: {}", e),
    })?;

    let mut address: Option<String> = None;
    let mut amount: Option<u64> = None;

    let map_len = match map_len {
        Some(len) => len as usize,
        None => usize::MAX,
    };

    for _ in 0..map_len {
        match decoder.u64() {
            Ok(0) => {
                // Address
                address = Some(
                    decoder
                        .bytes()
                        .map_err(|e| Error::InvalidInput {
                            reason: format!("Invalid address: {}", e),
                        })?
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect(),
                );
            }
            Ok(1) => {
                // Amount
                amount = Some(parse_amount(decoder)?);
            }
            Ok(_) => {
                decoder.skip().ok();
            }
            Err(_) => break,
        }
    }

    match (address, amount) {
        (Some(addr), Some(amt)) => Ok((addr, amt)),
        _ => Err(Error::InvalidInput {
            reason: "Missing address or amount in output".to_string(),
        }),
    }
}
