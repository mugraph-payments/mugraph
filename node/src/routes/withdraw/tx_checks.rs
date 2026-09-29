use std::collections::{BTreeMap, HashMap};

use mugraph_core::error::Error;
use whisky_csl::csl;

use super::ParsedWithdrawalTx;
use crate::{
    deposit_datum::{DepositDatumContext, parse_deposit_datum},
    network::CardanoNetwork,
};

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

/// Adds the quantities in `value` to `totals`, with the unit strings that
/// Cardano providers use: "lovelace", or the policy ID and the raw asset
/// name bytes in hex.
fn add_value_units(
    value: &csl::Value,
    totals: &mut BTreeMap<String, u128>,
) -> Result<(), Error> {
    let parse = |qty: String| {
        qty.parse::<u128>().map_err(|e| Error::InvalidInput {
            reason: format!("Invalid amount {}: {}", qty, e),
        })
    };

    *totals.entry("lovelace".to_string()).or_default() +=
        parse(value.coin().to_str())?;

    if let Some(ma) = value.multiasset() {
        let policies = ma.keys();
        for i in 0..policies.len() {
            let policy = policies.get(i);
            let Some(assets) = ma.get(&policy) else {
                continue;
            };
            let names = assets.keys();
            for j in 0..names.len() {
                let name = names.get(j);
                let Some(qty) = assets.get(&name) else {
                    continue;
                };
                let unit =
                    format!("{}{}", policy.to_hex(), hex::encode(name.name()));
                *totals.entry(unit).or_default() += parse(qty.to_str())?;
            }
        }
    }

    Ok(())
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
    let tx = parse_tx(tx_cbor)?;
    validate_transaction_balance_from_tx(&tx, input_totals, max_fee)
}

#[cfg(test)]
fn parse_tx(tx_cbor: &[u8]) -> Result<csl::Transaction, Error> {
    csl::Transaction::from_bytes(tx_cbor.to_vec()).map_err(|e| {
        Error::InvalidInput {
            reason: format!("Invalid transaction CBOR: {}", e),
        }
    })
}

/// Checks the ledger balance: inputs = outputs + fee, for each unit.
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

    let mut output_totals = BTreeMap::new();
    for output in &tx.body().outputs() {
        add_value_units(&output.amount(), &mut output_totals)?;
    }

    let in_lovelace = input_totals.get("lovelace").copied().unwrap_or(0);
    let out_lovelace = output_totals.get("lovelace").copied().unwrap_or(0);

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

    let units: std::collections::BTreeSet<&String> = input_totals
        .keys()
        .chain(output_totals.keys())
        .filter(|unit| *unit != "lovelace")
        .collect();
    for unit in units {
        let in_qty = input_totals.get(unit).copied().unwrap_or(0);
        let out_qty = output_totals.get(unit).copied().unwrap_or(0);
        if in_qty != out_qty {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Asset imbalance for {}: inputs {}, outputs {}",
                    unit, in_qty, out_qty
                ),
            });
        }
    }

    Ok(())
}

pub(super) fn validate_network_and_vault_outputs_with_parsed_tx(
    parsed_tx: &ParsedWithdrawalTx,
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<BTreeMap<String, u128>, Error> {
    validate_network_and_vault_outputs_from_tx(&parsed_tx.tx, wallet)
}

#[cfg(test)]
pub(super) fn validate_network_and_vault_outputs(
    tx_cbor: &[u8],
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<BTreeMap<String, u128>, Error> {
    validate_network_and_vault_outputs_from_tx(&parse_tx(tx_cbor)?, wallet)
}

/// Checks that each output is on the wallet's network, and that each
/// output to the vault has an inline datum for this node. Without that
/// datum, the vault validator can not spend the output, and the funds
/// are locked. Returns the total value that goes back to the vault.
fn validate_network_and_vault_outputs_from_tx(
    tx: &csl::Transaction,
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<BTreeMap<String, u128>, Error> {
    let expected_network_id = CardanoNetwork::parse(&wallet.network)
        .map_err(|e| Error::InvalidInput {
            reason: e.to_string(),
        })?
        .address_network_id();

    let node_pk_hash = csl::PublicKey::from_bytes(&wallet.payment_vk)
        .ok()
        .map(|pk| pk.hash().to_bytes());

    let mut vault_totals = BTreeMap::new();

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
        if bech32 != wallet.script_address {
            continue;
        }

        let datum =
            output.plutus_data().ok_or_else(|| Error::InvalidInput {
                reason: format!("Vault output {} has no inline datum", idx),
            })?;
        let datum = parse_deposit_datum(
            &hex::encode(datum.to_bytes()),
            DepositDatumContext::WithdrawalOutput { output_index: idx },
        )?;
        if Some(datum.node_pubkey_hash.to_vec()) != node_pk_hash {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Vault output {} node_pubkey_hash is not this node",
                    idx
                ),
            });
        }

        add_value_units(&output.amount(), &mut vault_totals)?;
    }

    Ok(vault_totals)
}

/// Checks that the notes pay for exactly what leaves the vault.
///
/// For each unit: burned notes - change notes = vault inputs - vault
/// outputs. The value that leaves the vault includes the transaction fee.
pub(super) fn validate_withdraw_value(
    vault_in: &BTreeMap<String, u128>,
    vault_out: &BTreeMap<String, u128>,
    notes: &BTreeMap<String, u128>,
    change: &BTreeMap<String, u128>,
) -> Result<(), Error> {
    let units: std::collections::BTreeSet<&String> = vault_in
        .keys()
        .chain(vault_out.keys())
        .chain(notes.keys())
        .chain(change.keys())
        .collect();

    for unit in units {
        let get = |m: &BTreeMap<String, u128>| {
            m.get(unit).copied().unwrap_or(0) as i128
        };
        let outflow = get(vault_in) - get(vault_out);
        let paid = get(notes) - get(change);

        if outflow != paid {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Notes value {} for {} does not match the vault outflow {}",
                    paid, unit, outflow
                ),
            });
        }
    }

    Ok(())
}
