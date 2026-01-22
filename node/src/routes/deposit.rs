use color_eyre::eyre::{Context as _, Result};
use mugraph_core::{
    error::Error,
    types::{DepositRequest, DepositResponse, Response, UtxoRef},
};

use crate::{
    cardano::setup_cardano_wallet,
    database::{CARDANO_WALLET, DEPOSITS, Database},
    routes::Context,
};

/// Handle deposit request
///
/// 1. Parse and validate the request payload
/// 2. Verify CIP-8 signature
/// 3. Fetch UTxO from provider
/// 4. Validate UTxO is at script address and unspent
/// 5. Map assets and validate amounts
/// 6. Sign blinded outputs
/// 7. Record deposit in database
pub async fn handle_deposit(request: &DepositRequest, ctx: &Context) -> Result<Response, Error> {
    tracing::info!(
        "Processing deposit request for UTxO: {}:{}",
        &request.utxo.tx_hash[..std::cmp::min(16, request.utxo.tx_hash.len())],
        request.utxo.index
    );

    // TODO: Implement the full deposit flow:
    // 1. Load or create Cardano wallet
    let wallet = load_or_create_wallet(ctx).await?;

    // 2. Verify CIP-8 signature over canonical payload
    verify_deposit_signature(request, &wallet)?;

    // 3. Fetch UTxO from Cardano provider and validate
    // validate_utxo(request, &wallet, ctx).await?;

    // 4. Validate outputs cover all assets in UTxO
    // validate_deposit_amounts(request)?;

    // 5. Sign blinded outputs with delegate key
    // let signatures = sign_outputs(request, &ctx.keypair)?;

    // 6. Record deposit in database
    // record_deposit(request, ctx).await?;

    // For now, return a placeholder response
    let deposit_ref = format!("{}:{}", request.utxo.tx_hash, request.utxo.index);

    Ok(Response::Deposit {
        signatures: vec![], // TODO: Return actual blind signatures
        deposit_ref,
    })
}

/// Load Cardano wallet from database or create new one
async fn load_or_create_wallet(ctx: &Context) -> Result<mugraph_core::types::CardanoWallet, Error> {
    // Try to load existing wallet
    {
        let read_tx = ctx.database.read()?;
        let table = read_tx.open_table(CARDANO_WALLET)?;
        if let Some(wallet_data) = table.get("wallet")? {
            return Ok(wallet_data.value());
        }
    }

    // Create new wallet if not found
    // TODO: Get network from config
    let wallet = setup_cardano_wallet("preprod", None)
        .await
        .map_err(|e| Error::Internal {
            reason: e.to_string(),
        })?;

    // Store wallet in database
    let write_tx = ctx.database.write()?;
    {
        let mut table = write_tx.open_table(CARDANO_WALLET)?;
        table.insert("wallet", &wallet)?;
    }
    write_tx.commit()?;

    Ok(wallet)
}

/// Verify CIP-8 signature over canonical deposit payload
fn verify_deposit_signature(
    _request: &DepositRequest,
    _wallet: &mugraph_core::types::CardanoWallet,
) -> Result<(), Error> {
    // TODO: Implement CIP-8 signature verification
    // Payload should be canonical JSON of: utxo + outputs + delegate pk + script address + nonce + network tag
    tracing::debug!("Verifying deposit signature...");
    Ok(())
}

/// Record deposit in database
async fn record_deposit(request: &DepositRequest, ctx: &Context) -> Result<(), Error> {
    use mugraph_core::types::DepositRecord;

    let write_tx = ctx.database.write()?;
    {
        let mut table = write_tx.open_table(DEPOSITS)?;

        let tx_hash = hex::decode(&request.utxo.tx_hash).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid tx_hash hex: {}", e),
        })?;
        let tx_hash_array: [u8; 32] = tx_hash.try_into().map_err(|_| Error::InvalidInput {
            reason: "tx_hash must be 32 bytes".to_string(),
        })?;

        let utxo_ref = UtxoRef::new(tx_hash_array, request.utxo.index);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // TODO: Get actual block height from provider
        let block_height = 0u64;
        // TODO: Get expiration from config
        let expires_at = now + (24 * 60 * 60); // 24 hours

        let record = DepositRecord::new(block_height, now, expires_at);
        table.insert(utxo_ref, &record)?;
    }
    write_tx.commit()?;

    tracing::info!("Deposit recorded successfully");
    Ok(())
}
