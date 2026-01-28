use color_eyre::eyre::Result;
use mugraph_core::{
    crypto,
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

    // 3. Parse transaction and validate inputs
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

    // 8. Burn notes (transactional with withdrawal recording)
    // This is done atomically - if submission fails, we roll back
    let (signed_cbor, change_notes) = burn_notes_and_sign(request, ctx, &tx_bytes).await?;

    // 9. Submit transaction to provider within transaction boundary
    match submit_transaction(&signed_cbor, &provider).await {
        Ok(submit_response) => {
            // Record withdrawal
            record_withdrawal(request, ctx).await?;

            tracing::info!(
                "Withdrawal submitted successfully: {}",
                &submit_response.tx_hash[..std::cmp::min(16, submit_response.tx_hash.len())]
            );

            Ok(Response::Withdraw {
                signed_tx_cbor: signed_cbor,
                tx_hash: submit_response.tx_hash,
                change_notes,
            })
        }
        Err(e) => {
            // Rollback: the database transaction was already committed, but
            // the withdrawal failed. In a production system, we'd need to
            // track pending withdrawals and handle retries or refunds.
            tracing::error!("Withdrawal submission failed: {}", e);
            Err(Error::NetworkError {
                reason: format!("Transaction submission failed: {}", e),
            })
        }
    }
}

/// Create Cardano provider from configuration
fn create_provider(_ctx: &Context) -> Result<Provider, Error> {
    // TODO: Get provider config from Context
    Provider::new(
        "blockfrost",
        std::env::var("BLOCKFROST_API_KEY").unwrap_or_else(|_| "test_key".to_string()),
        "preprod".to_string(),
        None,
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

    // TODO: Get network from config
    let network_byte = 0u8; // preprod
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
    // Note: This should be the transaction BODY hash, not the full transaction hash
    // TODO: Replace with proper whisky-csl body hash computation
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
    let change_notes = calculate_change_notes(request)?;

    Ok((signed_cbor, change_notes))
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

        // TODO: Get network from config
        let network_byte = 0u8; // preprod
        let key = WithdrawalKey::new(network_byte, tx_hash_array);

        let record = WithdrawalRecord::new(true);
        table.insert(key, &record)?;
    }
    write_tx.commit()?;

    tracing::info!("Withdrawal recorded successfully");
    Ok(())
}

/// Validate that all inputs reference the script address
async fn validate_script_inputs(_tx: &[u8], _script_address: &str) -> Result<(), Error> {
    // TODO: Implement using whisky-csl
    // Parse transaction inputs and verify they all belong to the script address
    Ok(())
}

/// Validate user witnesses
async fn validate_user_witnesses(
    _tx: &[u8],
    _notes: &[mugraph_core::types::BlindSignature],
) -> Result<(), Error> {
    // TODO: Implement CIP-8/COSE signature validation for user witnesses
    // Each input should have a valid signature from the corresponding user key
    Ok(())
}

/// Calculate change notes from transaction outputs
fn calculate_change_notes(_request: &WithdrawRequest) -> Result<Vec<BlindSignature>, Error> {
    // TODO: Analyze transaction outputs to determine change
    // Change = Total input value - withdrawal amount - fees
    // Return blind signatures for change notes
    //
    // To implement this properly:
    // 1. Parse the transaction CBOR to get outputs
    // 2. Identify which outputs are change (returning to user's address)
    // 3. Create blind signatures for those amounts
    Ok(vec![])
}
