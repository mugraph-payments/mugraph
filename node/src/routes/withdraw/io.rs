use color_eyre::eyre::Result;
use mugraph_core::error::Error;

use crate::{database::CARDANO_WALLET, provider::Provider, routes::Context};

/// Create Cardano provider from configuration
pub(super) fn create_provider(ctx: &Context) -> Result<Provider, Error> {
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

/// Load Cardano wallet for signing
pub(super) fn load_wallet(
    ctx: &Context,
) -> Result<mugraph_core::types::CardanoWallet, Error> {
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
pub(super) async fn submit_transaction(
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
