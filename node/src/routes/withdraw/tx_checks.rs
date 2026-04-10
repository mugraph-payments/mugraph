use std::collections::HashMap;

use blake2::Digest;
use mugraph_core::{error::Error, types::BlindSignature};
use whisky_csl::csl;

use super::ParsedWithdrawalTx;
use crate::network::CardanoNetwork;

pub(super) fn validate_parsed_fee(
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
    let tolerance_factor = 100 + tolerance_pct as u64;
    let max_acceptable_fee =
        max_fee_lovelace.saturating_mul(tolerance_factor) / 100;

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

fn extract_transaction_fee_from_tx(
    tx: &csl::Transaction,
) -> Result<u64, Error> {
    let fee_str = tx.body().fee().to_str();
    fee_str.parse::<u64>().map_err(|e| Error::InvalidInput {
        reason: format!("Failed to parse fee: {}", e),
    })
}

fn max_acceptable_fee(max_fee_lovelace: u64, tolerance_pct: u8) -> u64 {
    let tolerance_factor = 100 + tolerance_pct as u64;
    max_fee_lovelace.saturating_mul(tolerance_factor) / 100
}

pub(super) fn validate_transaction_balance_with_parsed_tx(
    parsed_tx: &ParsedWithdrawalTx,
    input_totals: &HashMap<String, u128>,
    max_fee: u64,
    fee_tolerance_pct: u8,
) -> Result<(), Error> {
    let effective_max_fee = max_acceptable_fee(max_fee, fee_tolerance_pct);
    validate_transaction_balance_from_tx(
        &parsed_tx.tx,
        input_totals,
        effective_max_fee,
    )
}

#[cfg(test)]
pub(super) fn validate_transaction_balance_with_tolerance(
    tx_cbor: &[u8],
    input_totals: &HashMap<String, u128>,
    max_fee: u64,
    fee_tolerance_pct: u8,
) -> Result<(), Error> {
    let effective_max_fee = max_acceptable_fee(max_fee, fee_tolerance_pct);
    validate_transaction_balance(tx_cbor, input_totals, effective_max_fee)
}

#[cfg(test)]
pub(super) fn validate_transaction_balance(
    tx_cbor: &[u8],
    input_totals: &HashMap<String, u128>,
    max_fee: u64,
) -> Result<(), Error> {
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec()).map_err(|e| {
        Error::InvalidInput {
            reason: format!("Invalid transaction CBOR: {}", e),
        }
    })?;

    validate_transaction_balance_from_tx(&tx, input_totals, max_fee)
}

fn validate_transaction_balance_from_tx(
    tx: &csl::Transaction,
    input_totals: &HashMap<String, u128>,
    max_fee: u64,
) -> Result<(), Error> {
    let fee_u128: u128 =
        tx.body()
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

    let mut output_totals: HashMap<String, u128> = HashMap::new();
    for output in &tx.body().outputs() {
        let coin = output.amount().coin();
        let entry = output_totals.entry("lovelace".to_string()).or_insert(0);
        *entry = entry.saturating_add(coin.to_str().parse::<u128>().map_err(
            |e| Error::InvalidInput {
                reason: format!("Invalid lovelace amount: {}", e),
            },
        )?);

        if let Some(ma) = output.amount().multiasset() {
            let policies = ma.keys();
            for idx in 0..policies.len() {
                let policy = policies.get(idx);
                if let Some(assets) = ma.get(&policy) {
                    let names = assets.keys();
                    for j in 0..names.len() {
                        let asset_name = names.get(j);
                        let qty = assets.get(&asset_name).unwrap();
                        let unit = format!(
                            "{}{}",
                            policy.to_hex(),
                            asset_name.to_hex()
                        );
                        let e = output_totals.entry(unit).or_insert(0);
                        *e = e.saturating_add(
                            qty.to_str().parse::<u128>().map_err(|e| {
                                Error::InvalidInput {
                                    reason: format!(
                                        "Invalid multiasset quantity: {}",
                                        e
                                    ),
                                }
                            })?,
                        );
                    }
                }
            }
        }
    }

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

pub(super) fn validate_withdraw_intent_metadata_with_parsed_tx(
    parsed_tx: &ParsedWithdrawalTx,
    network: &str,
) -> Result<(), Error> {
    validate_withdraw_intent_metadata_from_tx(&parsed_tx.tx, network)
}

#[cfg(test)]
pub(super) fn validate_withdraw_intent_metadata(
    tx_cbor: &[u8],
    network: &str,
) -> Result<(), Error> {
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec()).map_err(|e| {
        Error::InvalidInput {
            reason: format!("Invalid transaction CBOR: {}", e),
        }
    })?;

    validate_withdraw_intent_metadata_from_tx(&tx, network)
}

fn validate_withdraw_intent_metadata_from_tx(
    tx: &csl::Transaction,
    network: &str,
) -> Result<(), Error> {
    let expected_network =
        CardanoNetwork::parse(network).map_err(|e| Error::InvalidInput {
            reason: e.to_string(),
        })?;

    let aux = tx.auxiliary_data().ok_or_else(|| Error::InvalidInput {
        reason: "Transaction missing auxiliary data for intent binding"
            .to_string(),
    })?;

    let metadata = aux.metadata().ok_or_else(|| Error::InvalidInput {
        reason: "Auxiliary data missing metadata map".to_string(),
    })?;

    let label =
        csl::BigNum::from_str("1914").map_err(|e| Error::InvalidInput {
            reason: format!("Invalid metadatum label: {}", e),
        })?;
    let metadatum =
        metadata.get(&label).ok_or_else(|| Error::InvalidInput {
            reason: "Metadata label 1914 missing for intent binding"
                .to_string(),
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
                    network_ok = CardanoNetwork::parse(&n_txt).ok()
                        == Some(expected_network);
                }
            }
            "tx_body_hash" => {
                if let Ok(h_txt) = val.as_text() {
                    type Blake2b256 =
                        blake2::Blake2b<blake2::digest::consts::U32>;
                    let h = Blake2b256::digest(&tx.body().to_bytes());
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

pub(super) fn validate_network_and_change_outputs_with_parsed_tx(
    parsed_tx: &ParsedWithdrawalTx,
    wallet: &mugraph_core::types::CardanoWallet,
    change_outputs: &[BlindSignature],
) -> Result<(), Error> {
    validate_network_and_change_outputs_from_tx(
        &parsed_tx.tx,
        wallet,
        change_outputs,
    )
}

#[cfg(test)]
pub(super) fn validate_network_and_change_outputs(
    tx_cbor: &[u8],
    wallet: &mugraph_core::types::CardanoWallet,
    change_outputs: &[BlindSignature],
) -> Result<(), Error> {
    let tx = csl::Transaction::from_bytes(tx_cbor.to_vec()).map_err(|e| {
        Error::InvalidInput {
            reason: format!("Invalid transaction CBOR: {}", e),
        }
    })?;

    validate_network_and_change_outputs_from_tx(&tx, wallet, change_outputs)
}

fn validate_network_and_change_outputs_from_tx(
    tx: &csl::Transaction,
    wallet: &mugraph_core::types::CardanoWallet,
    change_outputs: &[BlindSignature],
) -> Result<(), Error> {
    let expected_network_id = CardanoNetwork::parse(&wallet.network)
        .map_err(|e| Error::InvalidInput {
            reason: e.to_string(),
        })?
        .address_network_id();

    let mut script_output_indexes = Vec::new();

    for (idx, output) in (&tx.body().outputs()).into_iter().enumerate() {
        let addr = output.address();

        let net_id = addr.network_id().map_err(|e| Error::InvalidInput {
            reason: format!(
                "Failed to read network id for output {}: {}",
                idx, e
            ),
        })?;
        if net_id != expected_network_id {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Output {} has network_id {} but wallet is {}",
                    idx, net_id, wallet.network
                ),
            });
        }

        let bech32 = addr.to_bech32(None).map_err(|e| Error::InvalidInput {
            reason: format!("Invalid output address: {}", e),
        })?;

        if bech32 == wallet.script_address {
            script_output_indexes.push(idx);
        }
    }

    if script_output_indexes.len() != change_outputs.len() {
        return Err(Error::InvalidInput {
            reason: format!(
                "Script change outputs must match request.change_outputs by count and transaction output order: found {} script outputs at indexes {:?}, but request provided {} change_outputs",
                script_output_indexes.len(),
                script_output_indexes,
                change_outputs.len()
            ),
        });
    }

    Ok(())
}
