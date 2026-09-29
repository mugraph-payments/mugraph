use std::collections::BTreeMap;

use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    keyset::is_denomination,
    types::{DepositRequest, PublicKey},
};
use whisky_csl::csl;

use super::{claims::DepositClaims, signature::compute_intent_hash};
use crate::{
    deposit_datum::{DepositDatumContext, parse_deposit_datum},
    provider::{Provider, UtxoInfo},
    routes::Context,
};

pub(super) async fn validate_deposit_source(
    request: &DepositRequest,
    claims: &DepositClaims,
    wallet: &mugraph_core::types::CardanoWallet,
    provider: &Provider,
    ctx: &Context,
    delegate_pk: &PublicKey,
) -> Result<(), Error> {
    let utxo_info =
        fetch_and_validate_utxo(request, wallet, provider, ctx).await?;
    validate_parsed_deposit_datum(
        request,
        claims,
        wallet,
        &utxo_info,
        delegate_pk,
    )?;
    validate_deposit_amounts(
        request,
        &utxo_info,
        ctx.config.min_deposit_value(),
    )?;
    Ok(())
}

/// Validate that the on-chain datum matches the expected user hash, node hash, and intent hash.
#[cfg(test)]
pub(super) fn validate_deposit_datum(
    request: &DepositRequest,
    wallet: &mugraph_core::types::CardanoWallet,
    utxo_info: &UtxoInfo,
    delegate_pk: &PublicKey,
) -> Result<(), Error> {
    let claims = super::claims::parse_deposit_claims(request)?;
    validate_parsed_deposit_datum(
        request,
        &claims,
        wallet,
        utxo_info,
        delegate_pk,
    )
}

pub(super) fn validate_parsed_deposit_datum(
    request: &DepositRequest,
    claims: &DepositClaims,
    wallet: &mugraph_core::types::CardanoWallet,
    utxo_info: &UtxoInfo,
    delegate_pk: &PublicKey,
) -> Result<(), Error> {
    let datum_hex =
        utxo_info
            .datum
            .as_ref()
            .ok_or_else(|| Error::InvalidInput {
                reason:
                    "UTxO missing inline datum; required for deposit validation"
                        .to_string(),
            })?;

    let datum =
        parse_deposit_datum(datum_hex, DepositDatumContext::DepositUtxo)?;

    let expected_user_hash: [u8; 28] =
        csl::PublicKey::from_bytes(&claims.user_pubkey)
            .map_err(|e| Error::InvalidKey {
                reason: format!("Invalid user public key: {}", e),
            })?
            .hash()
            .to_bytes()
            .try_into()
            .expect("Cardano key hashes are always 28 bytes");

    let expected_node_hash: [u8; 28] =
        csl::PublicKey::from_bytes(&wallet.payment_vk)
            .map_err(|e| Error::InvalidKey {
                reason: format!("Invalid node payment_vk: {}", e),
            })?
            .hash()
            .to_bytes()
            .try_into()
            .expect("Cardano key hashes are always 28 bytes");

    let expected_intent_hash =
        compute_intent_hash(request, delegate_pk, &wallet.script_address);

    if datum.user_pubkey_hash != expected_user_hash {
        return Err(Error::InvalidInput {
            reason:
                "Datum user_pubkey_hash does not match provided user_pubkey"
                    .to_string(),
        });
    }

    if datum.node_pubkey_hash != expected_node_hash {
        return Err(Error::InvalidInput {
            reason: "Datum node_pubkey_hash does not match this node"
                .to_string(),
        });
    }

    if datum.intent_hash != expected_intent_hash {
        return Err(Error::InvalidInput {
            reason: "Datum intent_hash does not match canonical payload"
                .to_string(),
        });
    }

    Ok(())
}

/// Fetch UTxO from provider and validate it's at the script address
pub(super) async fn fetch_and_validate_utxo(
    request: &DepositRequest,
    wallet: &mugraph_core::types::CardanoWallet,
    provider: &Provider,
    ctx: &Context,
) -> Result<UtxoInfo, Error> {
    let utxo_info = provider
        .get_utxo(&request.utxo.tx_hash, request.utxo.index)
        .await
        .map_err(|e| Error::NetworkError {
            reason: format!("Failed to fetch UTxO: {}", e),
        })?
        .ok_or_else(|| Error::InvalidInput {
            reason: "UTxO not found on chain".to_string(),
        })?;

    if utxo_info.address != wallet.script_address {
        return Err(Error::InvalidInput {
            reason: format!(
                "UTxO not at script address. Expected: {}, Got: {}",
                wallet.script_address, utxo_info.address
            ),
        });
    }

    let _tx_hash_array = crate::tx_ids::parse_tx_hash(&request.utxo.tx_hash)?;

    let tip = provider.get_tip().await.map_err(|e| Error::NetworkError {
        reason: format!(
            "Failed to get chain tip for confirm depth check: {}",
            e
        ),
    })?;

    let confirm_depth = ctx.config.deposit_confirm_depth();

    match utxo_info.block_height {
        Some(utxo_block_height) => {
            let current_height = tip.block_height;
            let blocks_confirmed =
                current_height.saturating_sub(utxo_block_height);

            tracing::info!(
                "UTxO {}:{} at block {} ({} blocks confirmed, need {})",
                &request.utxo.tx_hash[..16],
                request.utxo.index,
                utxo_block_height,
                blocks_confirmed,
                confirm_depth
            );

            if blocks_confirmed < confirm_depth {
                return Err(Error::InvalidInput {
                    reason: format!(
                        "UTxO not sufficiently confirmed. Block height: {}, Current: {}, Confirmed: {} blocks, Required: {} blocks",
                        utxo_block_height,
                        current_height,
                        blocks_confirmed,
                        confirm_depth
                    ),
                });
            }

            tracing::info!(
                "UTxO {}:{} confirmed with {} blocks (required: {})",
                &request.utxo.tx_hash[..16],
                request.utxo.index,
                blocks_confirmed,
                confirm_depth
            );
        }
        None => {
            tracing::warn!(
                "UTxO {}:{} block height not available from provider. Cannot verify confirm depth.",
                &request.utxo.tx_hash[..16],
                request.utxo.index
            );
            return Err(Error::InvalidInput {
                reason: "Cannot verify UTxO confirmation depth: block height not available"
                    .to_string(),
            });
        }
    }

    Ok(utxo_info)
}

/// Validate that the outputs hold the exact value of the UTxO.
///
/// Each output is one denomination, and for each asset the output amounts
/// must add up to the quantity in the UTxO. Thus, the notes that the node
/// signs hold the same value as the deposit.
pub(super) fn validate_deposit_amounts(
    request: &DepositRequest,
    utxo_info: &UtxoInfo,
    min_deposit_value: u64,
) -> Result<(), Error> {
    let mut utxo_assets: BTreeMap<String, u128> = BTreeMap::new();
    for asset in &utxo_info.amount {
        let amount = asset.quantity.parse::<u128>().map_err(|e| {
            Error::InvalidInput {
                reason: format!("Invalid asset quantity: {}", e),
            }
        })?;
        *utxo_assets.entry(asset.unit.clone()).or_default() += amount;
    }

    if request.outputs.is_empty() {
        return Err(Error::InvalidInput {
            reason: "No outputs provided for deposit".to_string(),
        });
    }

    let mut output_assets: BTreeMap<String, u128> = BTreeMap::new();
    for (i, output) in request.outputs.iter().enumerate() {
        if !is_denomination(output.amount) {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Output {} amount {} is not a power of two",
                    i, output.amount
                ),
            });
        }
        *output_assets
            .entry(output.asset().cardano_unit())
            .or_default() += output.amount as u128;
    }

    if output_assets != utxo_assets {
        return Err(Error::InvalidInput {
            reason: format!(
                "Output value {:?} does not match UTxO value {:?}",
                output_assets, utxo_assets
            ),
        });
    }

    let lovelace_amount = utxo_assets.get("lovelace").copied().unwrap_or(0);
    if lovelace_amount < min_deposit_value as u128 {
        return Err(Error::InvalidInput {
            reason: format!(
                "Deposit value {} lovelace below minimum {} lovelace",
                lovelace_amount, min_deposit_value
            ),
        });
    }

    tracing::info!(
        "Validated deposit: {} assets in UTxO ({} lovelace), {} outputs",
        utxo_assets.len(),
        lovelace_amount,
        request.outputs.len()
    );

    Ok(())
}
