use std::collections::{HashMap, HashSet};

use color_eyre::eyre::Result;
use mugraph_core::error::Error;
use whisky_csl::csl;

use super::ParsedWithdrawalTx;
use crate::{
    deposit_datum::{DepositDatumContext, parse_deposit_datum},
    provider::Provider,
};

fn extract_transaction_inputs_from_tx(
    tx: &csl::Transaction,
) -> Result<Vec<(Vec<u8>, u32)>, Error> {
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

#[cfg(test)]
pub(super) fn checked_output_index(
    index: u32,
    input_pos: usize,
) -> Result<u16, Error> {
    u16::try_from(index).map_err(|_| Error::InvalidInput {
        reason: format!(
            "Input {} has output index {} which exceeds u16::MAX",
            input_pos, index
        ),
    })
}

#[cfg(not(test))]
fn checked_output_index(index: u32, input_pos: usize) -> Result<u16, Error> {
    u16::try_from(index).map_err(|_| Error::InvalidInput {
        reason: format!(
            "Input {} has output index {} which exceeds u16::MAX",
            input_pos, index
        ),
    })
}

/// Checks the transaction signers.
///
/// Each witness that the transaction carries must be valid. The node's
/// key hash must be in `required_signers`, because the vault validator
/// only lets the node spend vault UTxOs. The node adds its own witness
/// after all checks pass, so the transaction can arrive without
/// witnesses.
pub(super) fn validate_signers_with_parsed_tx(
    parsed_tx: &ParsedWithdrawalTx,
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<(), Error> {
    let tx = &parsed_tx.tx;
    verify_witness_set(tx, &parsed_tx.tx_hash)?;

    let node_hash = csl::PublicKey::from_bytes(&wallet.payment_vk)
        .map_err(|e| Error::InvalidKey {
            reason: format!("Invalid node payment_vk: {}", e),
        })?
        .hash()
        .to_hex();

    let required: Vec<String> = tx
        .body()
        .required_signers()
        .map(|signers| signers.into_iter().map(|s| s.to_hex()).collect())
        .unwrap_or_default();

    if !required.contains(&node_hash) {
        return Err(Error::InvalidSignature {
            reason: format!(
                "Transaction required_signers must include the node key hash {}",
                node_hash
            ),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    Ok(())
}

fn verify_witness_set(
    tx: &csl::Transaction,
    body_hash_bytes: &[u8],
) -> Result<(), Error> {
    let witness_set = tx.witness_set();

    if let Some(vkeys) = witness_set.vkeys() {
        for (idx, witness) in (&vkeys).into_iter().enumerate() {
            let pk: csl::PublicKey = witness.vkey().public_key();
            if !pk.verify(body_hash_bytes, &witness.signature()) {
                return Err(Error::InvalidSignature {
                    reason: format!("VKey witness {} signature invalid", idx),
                    signature: mugraph_core::types::Signature::default(),
                });
            }
        }
    }

    if let Some(bootstraps) = witness_set.bootstraps() {
        for (idx, witness) in (&bootstraps).into_iter().enumerate() {
            let pk: csl::PublicKey = witness.vkey().public_key();
            if !pk.verify(body_hash_bytes, &witness.signature()) {
                return Err(Error::InvalidSignature {
                    reason: format!(
                        "Bootstrap witness {} signature invalid",
                        idx
                    ),
                    signature: mugraph_core::types::Signature::default(),
                });
            }
        }
    }

    Ok(())
}

pub(super) async fn validate_script_inputs_with_parsed_tx(
    parsed_tx: &ParsedWithdrawalTx,
    wallet: &mugraph_core::types::CardanoWallet,
    provider: &Provider,
) -> Result<(HashMap<String, u128>, Vec<mugraph_core::types::UtxoRef>), Error> {
    let inputs = extract_transaction_inputs_from_tx(&parsed_tx.tx)?;
    validate_script_inputs_with_extracted_inputs(inputs, wallet, provider).await
}

/// Checks that every input is a vault UTxO for this node, and adds up
/// their value.
///
/// The vault is one pool: any vault UTxO can pay for a withdrawal,
/// because the notes, not the depositor, prove the claim on the value.
async fn validate_script_inputs_with_extracted_inputs(
    inputs: Vec<(Vec<u8>, u32)>,
    wallet: &mugraph_core::types::CardanoWallet,
    provider: &Provider,
) -> Result<(HashMap<String, u128>, Vec<mugraph_core::types::UtxoRef>), Error> {
    use mugraph_core::types::UtxoRef;

    if inputs.is_empty() {
        return Err(Error::InvalidInput {
            reason: "Transaction has no inputs".to_string(),
        });
    }

    let mut totals: HashMap<String, u128> = HashMap::new();
    let mut consumed: Vec<UtxoRef> = Vec::new();
    let mut seen: HashSet<(Vec<u8>, u32)> = HashSet::new();

    let node_pk =
        csl::PublicKey::from_bytes(&wallet.payment_vk).map_err(|e| {
            Error::InvalidKey {
                reason: format!("Invalid node payment_vk: {}", e),
            }
        })?;
    let node_pk_hash: [u8; 28] = node_pk
        .hash()
        .to_bytes()
        .try_into()
        .expect("Cardano key hashes are always 28 bytes");

    for (i, (tx_hash_bytes, index)) in inputs.iter().enumerate() {
        if !seen.insert((tx_hash_bytes.clone(), *index)) {
            return Err(Error::InvalidInput {
                reason: format!("Input {} is listed more than once", i),
            });
        }

        let tx_hash = hex::encode(tx_hash_bytes);
        let output_index = checked_output_index(*index, i)?;

        let utxo_info = match provider.get_utxo(&tx_hash, output_index).await {
            Ok(Some(utxo_info)) => utxo_info,
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
        };

        if utxo_info.address != wallet.script_address {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Input {} ({}:{}) is not from script address. Expected {}, got {}",
                    i,
                    &tx_hash[..16],
                    index,
                    wallet.script_address,
                    utxo_info.address
                ),
            });
        }

        let datum_hex =
            utxo_info
                .datum
                .as_ref()
                .ok_or_else(|| Error::InvalidInput {
                    reason: format!(
                        "Input {} ({}:{}) missing inline datum",
                        i,
                        &tx_hash[..16],
                        index
                    ),
                })?;
        let datum = parse_deposit_datum(
            datum_hex,
            DepositDatumContext::WithdrawalInput { input_index: i },
        )?;
        if datum.node_pubkey_hash != node_pk_hash {
            return Err(Error::InvalidInput {
                reason: format!(
                    "Input {} node_pubkey_hash mismatch; expected our node, got {}",
                    i,
                    hex::encode(datum.node_pubkey_hash)
                ),
            });
        }

        for asset in &utxo_info.amount {
            let qty: u128 = asset.quantity.parse::<u128>().map_err(|e| {
                Error::InvalidInput {
                    reason: format!("Invalid asset quantity: {}", e),
                }
            })?;
            *totals.entry(asset.unit.clone()).or_insert(0) += qty;
        }

        let tx_hash_array: [u8; 32] = tx_hash_bytes
            .as_slice()
            .try_into()
            .map_err(|_| Error::InvalidInput {
                reason: format!("Invalid tx_hash length for input {}", i),
            })?;
        consumed.push(UtxoRef::new(tx_hash_array, output_index));
    }

    Ok((totals, consumed))
}
