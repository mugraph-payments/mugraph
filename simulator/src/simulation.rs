use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::{Result, eyre};
use mugraph_core::{
    builder::RefreshBuilder,
    crypto,
    types::{Asset, BlindSignature, DleqProofWithBlinding, Hash, Note, PublicKey, Refresh},
};
use rand::{Rng, rngs::StdRng};
use tokio::time::{MissedTickBehavior, interval};

use crate::types::*;

pub async fn bootstrap_wallets(
    nodes: &[SimNode],
    state: &mut AppState,
    assets: &[SimAsset],
    wallets: usize,
    notes_per_wallet: usize,
    amount_range: (u64, u64),
    rng: &mut StdRng,
) -> Result<()> {
    state.wallets = (0..wallets)
        .map(|id| Wallet {
            id,
            home_node: id % nodes.len(),
            ..Default::default()
        })
        .collect();

    for wallet_id in 0..wallets {
        let node = &nodes[wallet_id % nodes.len()];
        for asset in assets.iter() {
            for _ in 0..notes_per_wallet {
                let amount = rng.random_range(amount_range.0..=amount_range.1);
                let note = node
                    .client
                    .emit(asset.policy_id, asset.asset_name, amount)
                    .await?;
                state.log(format!(
                    "emit node={} wallet={} asset={} amount={amount}",
                    wallet_id % nodes.len(),
                    wallet_id,
                    asset.name
                ));

                let w = &mut state.wallets[wallet_id];
                let key = Asset {
                    policy_id: asset.policy_id,
                    asset_name: asset.asset_name,
                };
                w.notes.entry(key).or_default().push(note);
            }
        }
    }

    Ok(())
}

pub fn reserve_spendable_note(
    wallet: &mut Wallet,
    assets: &[SimAsset],
    rng: &mut StdRng,
) -> Option<(Asset, Note)> {
    let n = assets.len();
    let start = rng.random_range(0..n);
    for i in 0..n {
        let a = &assets[(start + i) % n];
        let key = Asset {
            policy_id: a.policy_id,
            asset_name: a.asset_name,
        };
        if let Some(notes) = wallet.notes.get_mut(&key)
            && let Some(pos) = notes.iter().position(|n| n.amount > 0)
        {
            return Some((key, notes.swap_remove(pos)));
        }
    }
    None
}

pub fn build_refresh(
    input_owner: usize,
    output_owner: usize,
    asset: Asset,
    input_note: Note,
    amount: u64,
) -> Result<(Refresh, Vec<usize>)> {
    let mut builder = RefreshBuilder::new().input(input_note.clone()).output(
        asset.policy_id,
        asset.asset_name,
        amount,
    );

    let mut owners = vec![output_owner];

    if input_note.amount > amount {
        builder = builder.output(
            asset.policy_id,
            asset.asset_name,
            input_note.amount - amount,
        );
        owners.push(input_owner);
    }

    Ok((builder.build()?, owners))
}

pub fn materialize_outputs(
    refresh: &Refresh,
    outputs: Vec<BlindSignature>,
    owners: &[usize],
    delegate: PublicKey,
) -> Result<Vec<(usize, Note)>> {
    let mut created = Vec::new();
    let mut output_iter = outputs.into_iter();
    for (atom_idx, atom) in refresh.atoms.iter().enumerate() {
        if refresh.is_input(atom_idx) {
            continue;
        }

        let signature = output_iter
            .next()
            .ok_or_else(|| eyre!("missing signature for output {}", atom_idx))?;

        let asset = refresh
            .asset_ids
            .get(atom.asset_id as usize)
            .ok_or_else(|| eyre!("invalid asset index {}", atom.asset_id))?;

        let commitment = atom.commitment(&refresh.asset_ids);
        let blinded_point = crypto::hash_to_curve(commitment.as_ref());
        if !crypto::verify_dleq_signature(
            &delegate,
            &blinded_point,
            &signature.signature,
            &signature.proof,
        )? {
            return Err(eyre!("invalid DLEQ proof for output {}", atom_idx));
        }

        if !crypto::verify(&delegate, commitment.as_ref(), signature.signature.0)? {
            return Err(eyre!("invalid signature for output {}", atom_idx));
        }

        let note = Note {
            amount: atom.amount,
            delegate: atom.delegate,
            policy_id: asset.policy_id,
            asset_name: asset.asset_name,
            nonce: atom.nonce,
            signature: signature.signature.0,
            dleq: Some(DleqProofWithBlinding {
                proof: signature.proof,
                blinding_factor: Hash::zero(),
            }),
        };

        let owner = owners
            .get(created.len())
            .ok_or_else(|| eyre!("missing owner mapping"))?;
        created.push((*owner, note));
    }

    Ok(created)
}

fn apply_successful_tx(state: &mut AppState, pending: &PendingTx, notes: Vec<(usize, Note)>) {
    for (owner, note) in notes {
        let key = Asset {
            policy_id: note.policy_id,
            asset_name: note.asset_name,
        };
        state.wallets[owner]
            .notes
            .entry(key)
            .or_default()
            .push(note);
    }

    state.wallets[pending.sender_id].sent += 1;
    state.wallets[pending.receiver_id].received += 1;
    state.total_ok += 1;
    state.log(format!(
        "tx {} ok sender={} receiver={} amount={}",
        pending.id, pending.sender_id, pending.receiver_id, pending.spend_amount
    ));
}

fn record_sender_failure(
    state: &mut AppState,
    sender_id: usize,
    asset: Asset,
    input_note: Note,
    message: String,
) {
    state.total_err += 1;
    state.wallets[sender_id].failures += 1;
    state.wallets[sender_id]
        .notes
        .entry(asset)
        .or_default()
        .push(input_note);
    state.log(&message);
    state.last_failure = Some(message);
}

pub async fn simulation_owner_loop(
    nodes: Vec<SimNode>,
    mut state: AppState,
    mut rng: StdRng,
    config: SimConfig,
    channels: SimChannels,
) {
    let SimChannels {
        mut cmd_rx,
        mut event_rx,
        event_tx,
        snapshot_tx,
    } = channels;

    let mut oracle = ConservationOracle::new();
    oracle.seal(&state);

    // Track per-asset amounts currently in-flight (removed from wallets, not yet returned)
    let mut inflight_amounts: HashMap<Asset, u128> = HashMap::new();

    let mut throughput = ThroughputTracker::new(Duration::from_secs(5));

    let mut ticker = interval(config.tick);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let max_inflight = config.max_inflight;

    let _ = snapshot_tx.send(state.snapshot(
        oracle.checks_passed(),
        max_inflight,
        throughput.tx_per_sec(),
        throughput.success_rate(),
    ));

    let mut tx_id: u64 = 0;

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if state.shutdown {
                    break;
                }
                if state.paused || state.inflight >= config.max_inflight {
                    continue;
                }

                let wallet_count = state.wallets.len();
                if wallet_count < 2 {
                    continue;
                }

                let sender_idx = rng.random_range(0..wallet_count);
                let receiver_idx = {
                    let mut idx = rng.random_range(0..wallet_count - 1);
                    if idx >= sender_idx {
                        idx += 1;
                    }
                    idx
                };

                let sender_id = state.wallets[sender_idx].id;
                let receiver_id = state.wallets[receiver_idx].id;

                let Some((asset, input_note)) = reserve_spendable_note(
                    &mut state.wallets[sender_idx],
                    &state.assets,
                    &mut rng,
                ) else {
                    continue;
                };

                let spend_amount = rng
                    .random_range(config.amount_range.0..=config.amount_range.1)
                    .min(input_note.amount);

                let input_amount = input_note.amount;

                let (refresh, owners) = match build_refresh(
                    sender_id,
                    receiver_id,
                    asset,
                    input_note.clone(),
                    spend_amount,
                ) {
                    Ok(res) => res,
                    Err(e) => {
                        record_sender_failure(
                            &mut state,
                            sender_id,
                            asset,
                            input_note,
                            format!("failed to build refresh: {e:#}"),
                        );
                        oracle.assert_conservation(
                            &state,
                            &inflight_amounts,
                            &format!("after build_refresh failure (tx_id={})", tx_id + 1),
                        );
                        let _ = snapshot_tx.send(state.snapshot(oracle.checks_passed(), max_inflight, throughput.tx_per_sec(), throughput.success_rate()));
                        continue;
                    }
                };

                // Track the reserved note's amount as inflight
                *inflight_amounts.entry(asset).or_default() += input_amount as u128;

                oracle.assert_conservation(
                    &state,
                    &inflight_amounts,
                    &format!("after reserve (tx_id={})", tx_id + 1),
                );

                // Route to the node that signed this note
                let note_delegate = input_note.delegate;
                let node = nodes
                    .iter()
                    .find(|n| n.delegate_pk == note_delegate)
                    .unwrap_or(&nodes[0]);

                tx_id += 1;
                let pending = PendingTx {
                    id: tx_id,
                    sender_id,
                    receiver_id,
                    asset,
                    input_amount,
                    input_note,
                    spend_amount,
                    refresh,
                    owners,
                    delegate: note_delegate,
                };

                state.inflight += 1;
                state.total_sent += 1;
                let _ = snapshot_tx.send(state.snapshot(oracle.checks_passed(), max_inflight, throughput.tx_per_sec(), throughput.success_rate()));

                let client_clone = node.client.clone();
                let event_tx_clone = event_tx.clone();
                tokio::spawn(async move {
                    let result = match client_clone.refresh(&pending.refresh).await {
                        Ok(outputs) => Ok(outputs),
                        Err(e) => Err(e.to_string()),
                    };
                    let _ = event_tx_clone.send(SimEvent::TxFinished { pending, result });
                });
            }
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    SimCommand::TogglePause => {
                        state.paused = !state.paused;
                        state.log(format!("paused set to {}", state.paused));
                        let _ = snapshot_tx.send(state.snapshot(oracle.checks_passed(), max_inflight, throughput.tx_per_sec(), throughput.success_rate()));
                    }
                    SimCommand::Quit => {
                        state.shutdown = true;
                        state.log(format!(
                            "shutting down — conservation checks passed: {}",
                            oracle.checks_passed(),
                        ));
                        let _ = snapshot_tx.send(state.snapshot(oracle.checks_passed(), max_inflight, throughput.tx_per_sec(), throughput.success_rate()));
                        break;
                    }
                }
            }
            Some(event) = event_rx.recv() => {
                match event {
                    SimEvent::TxFinished { pending, result } => {
                        state.inflight = state.inflight.saturating_sub(1);

                        // Remove inflight tracking for this tx
                        let entry = inflight_amounts.entry(pending.asset).or_default();
                        *entry = entry.saturating_sub(pending.input_amount as u128);

                        match result {
                            Ok(outputs) => match materialize_outputs(
                                &pending.refresh,
                                outputs,
                                &pending.owners,
                                pending.delegate,
                            ) {
                                Ok(notes) => {
                                    throughput.record_ok();
                                    apply_successful_tx(&mut state, &pending, notes);
                                    oracle.assert_conservation(
                                        &state,
                                        &inflight_amounts,
                                        &format!("after successful tx {}", pending.id),
                                    );
                                }
                                Err(e) => {
                                    throughput.record_err();
                                    record_sender_failure(
                                        &mut state,
                                        pending.sender_id,
                                        pending.asset,
                                        pending.input_note,
                                        format!("tx {} materialization failed: {e:#}", pending.id),
                                    );
                                    oracle.assert_conservation(
                                        &state,
                                        &inflight_amounts,
                                        &format!("after materialization failure tx {}", pending.id),
                                    );
                                }
                            },
                            Err(reason) => {
                                throughput.record_err();
                                record_sender_failure(
                                    &mut state,
                                    pending.sender_id,
                                    pending.asset,
                                    pending.input_note,
                                    format!("tx {} failed: {}", pending.id, reason),
                                );
                                oracle.assert_conservation(
                                    &state,
                                    &inflight_amounts,
                                    &format!("after rpc failure tx {}", pending.id),
                                );
                            }
                        }
                        let _ = snapshot_tx.send(state.snapshot(oracle.checks_passed(), max_inflight, throughput.tx_per_sec(), throughput.success_rate()));
                    }
                }
            }
        }
    }

    state.shutdown = true;
    let _ = snapshot_tx.send(state.snapshot(
        oracle.checks_passed(),
        max_inflight,
        throughput.tx_per_sec(),
        throughput.success_rate(),
    ));
}

#[cfg(test)]
mod tests {
    use mugraph_core::types::{AssetName, Hash, PolicyId, Signature};
    use proptest::prelude::*;

    use super::*;

    fn dummy_note(signature_byte: u8) -> Note {
        Note {
            amount: 1,
            delegate: PublicKey([9u8; 32]),
            policy_id: PolicyId([7u8; 28]),
            asset_name: AssetName::empty(),
            nonce: Hash([5u8; 32]),
            signature: Signature([signature_byte; 32]),
            dleq: None,
        }
    }

    fn note_with_amount(amount: u64) -> Note {
        Note {
            amount,
            delegate: PublicKey([9u8; 32]),
            policy_id: PolicyId([7u8; 28]),
            asset_name: AssetName::empty(),
            nonce: Hash([5u8; 32]),
            signature: Signature([1u8; 32]),
            dleq: None,
        }
    }

    #[test]
    fn reserve_by_signature_survives_reordering() {
        let mut notes = vec![dummy_note(1), dummy_note(2)];
        let target = notes[1].clone();

        notes.swap_remove(0);

        let Some(pos) = notes.iter().position(|n| n.signature == target.signature) else {
            panic!("target note missing");
        };

        notes.swap_remove(pos);
        assert!(notes.is_empty());
    }

    #[test]
    fn reinsert_is_deduped_by_signature() {
        let mut notes = vec![dummy_note(3)];
        let note = notes[0].clone();

        if !notes.iter().any(|n| n.signature == note.signature) {
            notes.push(note.clone());
        }
        assert_eq!(notes.len(), 1);

        let other = dummy_note(4);
        if !notes.iter().any(|n| n.signature == other.signature) {
            notes.push(other);
        }
        assert_eq!(notes.len(), 2);
    }

    proptest! {
        #[test]
        fn prop_build_refresh_conserves_total_amount(
            (input_amount, spend_amount) in (1u64..=1_000_000)
                .prop_flat_map(|input_amount| (Just(input_amount), 1u64..=input_amount)),
            sender in 0usize..16,
            receiver in 0usize..16,
        ) {
            let asset = Asset {
                policy_id: PolicyId([7u8; 28]),
                asset_name: AssetName::empty(),
            };
            let input_note = note_with_amount(input_amount);

            let (refresh, owners) = build_refresh(
                sender,
                receiver,
                asset,
                input_note,
                spend_amount,
            ).unwrap();

            let output_total: u64 = refresh
                .atoms
                .iter()
                .enumerate()
                .filter(|(idx, _)| !refresh.is_input(*idx))
                .map(|(_, atom)| atom.amount)
                .sum();

            prop_assert_eq!(output_total, input_amount);

            if spend_amount < input_amount {
                prop_assert_eq!(owners.len(), 2);
                prop_assert_eq!(owners[0], receiver);
                prop_assert_eq!(owners[1], sender);
            } else {
                prop_assert_eq!(owners.len(), 1);
                prop_assert_eq!(owners[0], receiver);
            }
        }

        #[test]
        fn prop_build_refresh_outputs_never_exceed_input(
            (input_amount, spend_amount) in (1u64..=500_000)
                .prop_flat_map(|input_amount| (Just(input_amount), 1u64..=input_amount)),
        ) {
            let asset = Asset {
                policy_id: PolicyId([7u8; 28]),
                asset_name: AssetName::empty(),
            };

            let (refresh, _owners) = build_refresh(
                1,
                2,
                asset,
                note_with_amount(input_amount),
                spend_amount,
            ).unwrap();

            for (idx, atom) in refresh.atoms.iter().enumerate() {
                if !refresh.is_input(idx) {
                    prop_assert!(atom.amount <= input_amount);
                }
            }
        }
    }
}
