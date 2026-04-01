use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{DepositRequest, UtxoRef},
};
use redb::ReadableTable;

use super::signature::compute_intent_hash;
use crate::{
    cardano::setup_cardano_wallet,
    database::{CARDANO_WALLET, DEPOSITS},
    provider::Provider,
    routes::Context,
};

/// Load Cardano wallet from database or create new one
pub(super) async fn load_or_create_wallet(
    ctx: &Context,
) -> Result<mugraph_core::types::CardanoWallet, Error> {
    {
        let read_tx = ctx.database.read()?;
        let table = read_tx.open_table(CARDANO_WALLET)?;
        if let Some(wallet_data) = table.get("wallet")? {
            return Ok(wallet_data.value());
        }
    }

    let network = ctx
        .config
        .cardano_network()
        .map(|network| network.as_str().to_string())
        .unwrap_or_else(|_| ctx.config.network());
    let payment_sk = ctx.config.payment_sk();

    let wallet = setup_cardano_wallet(&network, payment_sk.as_deref())
        .await
        .map_err(|e| Error::Internal {
            reason: e.to_string(),
        })?;

    let selected = store_wallet_if_absent(ctx, wallet)?;

    Ok(selected)
}

pub(super) fn store_wallet_if_absent(
    ctx: &Context,
    candidate: mugraph_core::types::CardanoWallet,
) -> Result<mugraph_core::types::CardanoWallet, Error> {
    let write_tx = ctx.database.write()?;
    let selected = {
        let mut table = write_tx.open_table(CARDANO_WALLET)?;
        if let Some(existing) = table.get("wallet")? {
            existing.value()
        } else {
            table.insert("wallet", &candidate)?;
            candidate
        }
    };
    write_tx.commit()?;
    Ok(selected)
}

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

pub(super) async fn persist_deposit(
    request: &DepositRequest,
    ctx: &Context,
    provider: &Provider,
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<String, Error> {
    record_deposit(request, ctx, provider, wallet).await?;
    Ok(format!("{}:{}", request.utxo.tx_hash, request.utxo.index))
}

pub(super) fn insert_deposit_if_absent(
    table: &mut redb::Table<'_, UtxoRef, mugraph_core::types::DepositRecord>,
    utxo_ref: UtxoRef,
    record: mugraph_core::types::DepositRecord,
) -> Result<(), Error> {
    if table.get(&utxo_ref)?.is_some() {
        return Err(Error::InvalidInput {
            reason: "Deposit already processed".to_string(),
        });
    }

    table.insert(utxo_ref, &record)?;
    Ok(())
}

/// Record deposit in database
pub(super) async fn record_deposit(
    request: &DepositRequest,
    ctx: &Context,
    provider: &Provider,
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<(), Error> {
    use mugraph_core::types::DepositRecord;

    let tip = provider.get_tip().await.map_err(|e| Error::NetworkError {
        reason: format!("Failed to get chain tip: {}", e),
    })?;

    let intent_hash = compute_intent_hash(
        request,
        &ctx.keypair.public_key,
        &wallet.script_address,
    );

    let write_tx = ctx.database.write()?;
    {
        let mut table = write_tx.open_table(DEPOSITS)?;

        let utxo_ref = crate::tx_ids::parse_utxo_ref(
            &request.utxo.tx_hash,
            request.utxo.index,
        )?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let expiration_seconds = ctx.config.deposit_expiration_blocks() * 20;
        let expires_at = now + expiration_seconds;

        let record = DepositRecord::with_intent_hash(
            tip.block_height,
            now,
            expires_at,
            intent_hash,
        );
        insert_deposit_if_absent(&mut table, utxo_ref, record)?;
    }
    write_tx.commit()?;

    tracing::info!(
        "Deposit recorded successfully at block {}",
        tip.block_height
    );
    Ok(())
}
