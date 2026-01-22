use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{Response, WithdrawRequest, WithdrawalKey, WithdrawalRecord},
};

use crate::{
    database::{Database, WITHDRAWALS},
    routes::Context,
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

    // TODO: Implement the full withdrawal flow:

    // 1. Preflight validation
    // preflight_withdrawal(request, ctx).await?;

    // 2. Check idempotency via WITHDRAWALS table
    // check_idempotency(request, ctx).await?;

    // 3. Burn notes (transactional)
    // burn_notes(request, ctx).await?;

    // 4. Attach node witness
    // let signed_cbor = attach_node_witness(request, ctx).await?;

    // 5. Submit to provider
    // submit_transaction(&signed_cbor, ctx).await?;

    // 6. Record withdrawal
    // record_withdrawal(request, ctx).await?;

    // 7. Calculate change notes from transaction outputs
    // let change_notes = calculate_change_notes(request)?;

    // For now, return a placeholder response
    Ok(Response::Withdraw {
        signed_tx_cbor: request.tx_cbor.clone(), // TODO: Return actually signed CBOR
        tx_hash: request.tx_hash.clone(),
        change_notes: vec![], // TODO: Return actual change notes
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

/// Preflight validation of withdrawal transaction
async fn preflight_withdrawal(request: &WithdrawRequest, _ctx: &Context) -> Result<(), Error> {
    // 1. Parse tx_cbor and recompute hash
    let tx_bytes = hex::decode(&request.tx_cbor).map_err(|e| Error::InvalidInput {
        reason: format!("Invalid tx_cbor hex: {}", e),
    })?;

    // TODO: Use whisky-csl to parse transaction
    tracing::debug!("Transaction size: {} bytes", tx_bytes.len());

    // TODO: Verify provided hash matches recomputed hash

    // TODO: Ensure all inputs reference script UTxOs

    // TODO: Validate user signatures (CIP-8/COSE)

    // TODO: Check outputs match burned notes minus fees

    Ok(())
}
