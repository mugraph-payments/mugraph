use std::collections::{HashMap, HashSet};

use color_eyre::eyre::Result;
use mugraph_core::{error::Error, types::BlindSignature};
use whisky_csl::csl;

use super::ParsedWithdrawalTx;
use crate::{
    deposit_datum::{DepositDatumContext, parse_deposit_datum},
    provider::Provider,
    routes::Context,
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

pub(super) async fn validate_user_witnesses_with_parsed_tx(
    parsed_tx: &ParsedWithdrawalTx,
    notes: &[BlindSignature],
    expected_user_hashes: &HashSet<String>,
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<(), Error> {
    validate_user_witnesses_from_tx(
        &parsed_tx.tx,
        &parsed_tx.tx_hash,
        notes,
        expected_user_hashes,
        wallet,
    )
    .await
}

#[cfg(test)]
pub(super) async fn validate_user_witnesses(
    tx_cbor: &[u8],
    notes: &[BlindSignature],
    expected_user_hashes: &HashSet<String>,
    wallet: &mugraph_core::types::CardanoWallet,
) -> Result<(), Error> {
    let parsed_tx = ParsedWithdrawalTx::parse(&hex::encode(tx_cbor))?;
    validate_user_witnesses_with_parsed_tx(
        &parsed_tx,
        notes,
        expected_user_hashes,
        wallet,
    )
    .await
}

async fn validate_user_witnesses_from_tx(
    tx: &csl::Transaction,
    tx_hash: &[u8; 32],
    notes: &[BlindSignature],
    expected_user_hashes: &HashSet<String>,
    _wallet: &mugraph_core::types::CardanoWallet,
) -> Result<(), Error> {
    let body_hash_bytes = tx_hash.to_vec();

    let (witness_key_hashes, verified_witnesses) =
        verify_witness_set(tx, &body_hash_bytes)?;
    let required_signer_hashes = collect_required_signer_hashes(tx)?;

    ensure_required_signers_have_witnesses(
        &required_signer_hashes,
        &witness_key_hashes,
    )?;
    ensure_expected_owner_hashes_are_bound(
        expected_user_hashes,
        &required_signer_hashes,
        &witness_key_hashes,
    )?;

    tracing::info!(
        "Validated {} witness signatures for {} notes",
        verified_witnesses,
        notes.len()
    );

    Ok(())
}

fn verify_witness_set(
    tx: &csl::Transaction,
    body_hash_bytes: &[u8],
) -> Result<(HashSet<String>, usize), Error> {
    let witness_set = tx.witness_set();
    let mut verified_witnesses = 0usize;
    let mut witness_key_hashes = HashSet::new();

    if let Some(vkeys) = witness_set.vkeys() {
        for (idx, witness) in (&vkeys).into_iter().enumerate() {
            let pk: csl::PublicKey = witness.vkey().public_key();
            let sig = witness.signature();
            if !pk.verify(body_hash_bytes, &sig) {
                return Err(Error::InvalidSignature {
                    reason: format!("VKey witness {} signature invalid", idx),
                    signature: mugraph_core::types::Signature::default(),
                });
            }
            witness_key_hashes.insert(pk.hash().to_hex());
            verified_witnesses += 1;
        }
    }

    if let Some(bootstraps) = witness_set.bootstraps() {
        for (idx, witness) in (&bootstraps).into_iter().enumerate() {
            let pk: csl::PublicKey = witness.vkey().public_key();
            let sig = witness.signature();
            if !pk.verify(body_hash_bytes, &sig) {
                return Err(Error::InvalidSignature {
                    reason: format!(
                        "Bootstrap witness {} signature invalid",
                        idx
                    ),
                    signature: mugraph_core::types::Signature::default(),
                });
            }
            witness_key_hashes.insert(pk.hash().to_hex());
            verified_witnesses += 1;
        }
    }

    if verified_witnesses == 0 {
        return Err(Error::InvalidSignature {
            reason: "No valid witnesses found in transaction".to_string(),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    Ok((witness_key_hashes, verified_witnesses))
}

fn collect_required_signer_hashes(
    tx: &csl::Transaction,
) -> Result<Vec<String>, Error> {
    let required = tx.body().required_signers().ok_or_else(|| Error::InvalidSignature {
        reason: "Transaction missing required_signers; cannot bind witnesses to note owners"
            .to_string(),
        signature: mugraph_core::types::Signature::default(),
    })?;

    Ok(required.into_iter().map(|signer| signer.to_hex()).collect())
}

fn ensure_required_signers_have_witnesses(
    required_signer_hashes: &[String],
    witness_key_hashes: &HashSet<String>,
) -> Result<(), Error> {
    let missing: Vec<String> = required_signer_hashes
        .iter()
        .filter(|signer_hash| !witness_key_hashes.contains(*signer_hash))
        .cloned()
        .collect();

    if missing.is_empty() {
        return Ok(());
    }

    Err(Error::InvalidSignature {
        reason: format!(
            "Missing witnesses for required_signers: {:?}",
            missing
        ),
        signature: mugraph_core::types::Signature::default(),
    })
}

fn ensure_expected_owner_hashes_are_bound(
    expected_user_hashes: &HashSet<String>,
    required_signer_hashes: &[String],
    witness_key_hashes: &HashSet<String>,
) -> Result<(), Error> {
    if expected_user_hashes.is_empty() {
        return Err(Error::InvalidSignature {
            reason: "No expected user hashes derived from inputs".to_string(),
            signature: mugraph_core::types::Signature::default(),
        });
    }

    for expected in expected_user_hashes {
        if !required_signer_hashes
            .iter()
            .any(|signer_hash| signer_hash == expected)
        {
            return Err(Error::InvalidSignature {
                reason: format!(
                    "Required signer set does not include input owner hash {}",
                    expected
                ),
                signature: mugraph_core::types::Signature::default(),
            });
        }

        if !witness_key_hashes.contains(expected) {
            return Err(Error::InvalidSignature {
                reason: format!(
                    "Missing witness for input owner hash {}",
                    expected
                ),
                signature: mugraph_core::types::Signature::default(),
            });
        }
    }

    Ok(())
}

pub(super) async fn validate_script_inputs_with_parsed_tx(
    parsed_tx: &ParsedWithdrawalTx,
    wallet: &mugraph_core::types::CardanoWallet,
    ctx: &Context,
    provider: &Provider,
) -> Result<
    (
        HashMap<String, u128>,
        HashSet<String>,
        Vec<mugraph_core::types::UtxoRef>,
    ),
    Error,
> {
    let inputs = extract_transaction_inputs_from_tx(&parsed_tx.tx)?;
    validate_script_inputs_with_extracted_inputs(inputs, wallet, ctx, provider)
        .await
}

async fn validate_script_inputs_with_extracted_inputs(
    inputs: Vec<(Vec<u8>, u32)>,
    wallet: &mugraph_core::types::CardanoWallet,
    ctx: &Context,
    provider: &Provider,
) -> Result<
    (
        HashMap<String, u128>,
        HashSet<String>,
        Vec<mugraph_core::types::UtxoRef>,
    ),
    Error,
> {
    use mugraph_core::types::UtxoRef;

    use crate::database::DEPOSITS;

    if inputs.is_empty() {
        return Err(Error::InvalidInput {
            reason: "Transaction has no inputs".to_string(),
        });
    }

    let mut totals: HashMap<String, u128> = HashMap::new();
    let mut required_user_hashes: HashSet<String> = HashSet::new();
    let mut consumed_deposits: Vec<UtxoRef> = Vec::new();
    let read_tx = ctx.database.read()?;
    let deposits_table = read_tx.open_table(DEPOSITS)?;

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
        let tx_hash = hex::encode(tx_hash_bytes);
        let output_index = checked_output_index(*index, i)?;

        tracing::debug!("Validating input {}: {}:{}", i, &tx_hash[..16], index);

        match provider.get_utxo(&tx_hash, output_index).await {
            Ok(Some(utxo_info)) => {
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

                let datum_hex = utxo_info
                    .datum
                    .as_ref()
                    .ok_or_else(|| Error::InvalidInput {
                        reason: format!(
                            "Input {} ({}:{}) missing inline datum; required for witness binding",
                            i,
                            &tx_hash[..16],
                            index
                        ),
                    })?;

                let datum = parse_deposit_datum(
                    datum_hex,
                    DepositDatumContext::WithdrawalInput { input_index: i },
                )?;

                required_user_hashes
                    .insert(hex::encode(datum.user_pubkey_hash));

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
                    let qty: u128 =
                        asset.quantity.parse::<u128>().map_err(|e| {
                            Error::InvalidInput {
                                reason: format!(
                                    "Invalid asset quantity: {}",
                                    e
                                ),
                            }
                        })?;
                    let entry = totals.entry(asset.unit.clone()).or_insert(0);
                    *entry = entry.saturating_add(qty);
                }

                let tx_hash_array: [u8; 32] = tx_hash_bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| Error::InvalidInput {
                        reason: format!(
                            "Invalid tx_hash length for input {}",
                            i
                        ),
                    })?;
                let utxo_ref = UtxoRef::new(tx_hash_array, output_index);
                consumed_deposits.push(utxo_ref.clone());

                match deposits_table.get(&utxo_ref)? {
                    Some(deposit) => {
                        let deposit_record = deposit.value();
                        if deposit_record.spent {
                            return Err(Error::InvalidInput {
                                reason: format!(
                                    "Input {} ({}:{}) deposit already spent",
                                    i,
                                    &tx_hash[..16],
                                    index
                                ),
                            });
                        }

                        if deposit_record.intent_hash != [0u8; 32]
                            && datum.intent_hash != deposit_record.intent_hash
                        {
                            return Err(Error::InvalidInput {
                                reason: format!(
                                    "Intent hash mismatch for input {}: datum {}, expected {}",
                                    i,
                                    hex::encode(datum.intent_hash),
                                    hex::encode(deposit_record.intent_hash)
                                ),
                            });
                        }

                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        if now > deposit_record.expires_at {
                            return Err(Error::InvalidInput {
                                reason: format!(
                                    "Input {} ({}:{}) deposit expired at {}",
                                    i,
                                    &tx_hash[..16],
                                    index,
                                    deposit_record.expires_at
                                ),
                            });
                        }

                        tracing::info!(
                            "Input {}: deposit valid (block {}, expires {})",
                            i,
                            deposit_record.block_height,
                            deposit_record.expires_at
                        );
                    }
                    None => {
                        tracing::warn!(
                            "Input {} ({}:{}) not found in DEPOSITS table",
                            i,
                            &tx_hash[..16],
                            index
                        );
                        return Err(Error::InvalidInput {
                            reason: format!(
                                "Input {} ({}:{}) deposit not found. Deposits must be recorded before withdrawal.",
                                i,
                                &tx_hash[..16],
                                index
                            ),
                        });
                    }
                }
            }
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
        }
    }

    tracing::info!(
        "Aggregated input totals: {:?}",
        totals
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<_>>()
            .join(", ")
    );

    Ok((totals, required_user_hashes, consumed_deposits))
}
