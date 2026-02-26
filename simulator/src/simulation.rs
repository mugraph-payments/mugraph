use std::time::Duration;

use color_eyre::eyre::{Result, eyre};
use mugraph_core::{
    builder::RefreshBuilder,
    crypto,
    types::{Asset, BlindSignature, DleqProofWithBlinding, Hash, Note, PublicKey, Refresh},
};
use rand::{Rng, rngs::StdRng, seq::SliceRandom};
use tokio::{
    sync::{mpsc, watch},
    time::{MissedTickBehavior, interval},
};

use crate::{client::NodeClient, types::*};

pub async fn bootstrap_wallets(
    client: &NodeClient,
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
            ..Default::default()
        })
        .collect();

    for wallet_id in 0..wallets {
        for asset in assets.iter() {
            for _ in 0..notes_per_wallet {
                let amount = rng.random_range(amount_range.0..=amount_range.1);
                let note = client.emit(asset.policy_id, asset.asset_name, amount).await?;
                state.log(format!(
                    "emit via node wallet={} asset={} amount={amount}",
                    wallet_id, asset.name
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
    let mut shuffled: Vec<Asset> = assets
        .iter()
        .map(|a| Asset {
            policy_id: a.policy_id,
            asset_name: a.asset_name,
        })
        .collect();
    shuffled.shuffle(rng);
    for asset in shuffled {
        if let Some(notes) = wallet.notes.get_mut(&asset)
            && let Some(pos) = notes.iter().position(|n| n.amount > 0)
        {
            return Some((asset, notes.swap_remove(pos)));
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
    let mut builder = RefreshBuilder::new().input(input_note.clone());
    builder = builder.output(asset.policy_id, asset.asset_name, amount);

    if input_note.amount > amount {
        let change = input_note.amount - amount;
        builder = builder.output(asset.policy_id, asset.asset_name, change);
    }

    let refresh = builder.build()?;

    let mut owners = Vec::new();
    owners.push(output_owner);
    if input_note.amount > amount {
        owners.push(input_owner);
    }

    Ok((refresh, owners))
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
        state.wallets[owner].notes.entry(key).or_default().push(note);
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
    state.last_failure = Some(message.clone());
    state.total_err += 1;
    state.wallets[sender_id].failures += 1;
    state.wallets[sender_id]
        .notes
        .entry(asset)
        .or_default()
        .push(input_note);
    state.log(message);
}

#[allow(clippy::too_many_arguments)]
pub async fn simulation_owner_loop(
    client: NodeClient,
    mut state: AppState,
    mut rng: StdRng,
    amount_range: (u64, u64),
    tick: Duration,
    max_inflight: usize,
    mut cmd_rx: mpsc::UnboundedReceiver<SimCommand>,
    mut event_rx: mpsc::UnboundedReceiver<SimEvent>,
    event_tx: mpsc::UnboundedSender<SimEvent>,
    snapshot_tx: watch::Sender<AppSnapshot>,
) {
    let mut ticker = interval(tick);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let _ = snapshot_tx.send(state.snapshot());

    let mut tx_id: u64 = 0;

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if state.shutdown {
                    break;
                }
                if state.paused || state.inflight >= max_inflight {
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
                    .random_range(amount_range.0..=amount_range.1)
                    .min(input_note.amount);

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
                        let _ = snapshot_tx.send(state.snapshot());
                        continue;
                    }
                };

                tx_id += 1;
                let pending = PendingTx {
                    id: tx_id,
                    sender_id,
                    receiver_id,
                    asset,
                    input_note,
                    spend_amount,
                    refresh,
                    owners,
                };

                state.inflight += 1;
                state.total_sent += 1;
                let _ = snapshot_tx.send(state.snapshot());

                let client_clone = client.clone();
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
                        let _ = snapshot_tx.send(state.snapshot());
                    }
                    SimCommand::Quit => {
                        state.shutdown = true;
                        state.log("shutting down");
                        let _ = snapshot_tx.send(state.snapshot());
                        break;
                    }
                }
            }
            Some(event) = event_rx.recv() => {
                match event {
                    SimEvent::TxFinished { pending, result } => {
                        state.inflight = state.inflight.saturating_sub(1);
                        match result {
                            Ok(outputs) => match materialize_outputs(
                                &pending.refresh,
                                outputs,
                                &pending.owners,
                                state.delegate_pk,
                            ) {
                                Ok(notes) => {
                                    apply_successful_tx(&mut state, &pending, notes);
                                }
                                Err(e) => {
                                    record_sender_failure(
                                        &mut state,
                                        pending.sender_id,
                                        pending.asset,
                                        pending.input_note,
                                        format!("tx {} materialization failed: {e:#}", pending.id),
                                    );
                                }
                            },
                            Err(reason) => {
                                record_sender_failure(
                                    &mut state,
                                    pending.sender_id,
                                    pending.asset,
                                    pending.input_note,
                                    format!("tx {} failed: {}", pending.id, reason),
                                );
                            }
                        }
                        let _ = snapshot_tx.send(state.snapshot());
                    }
                }
            }
        }
    }

    state.shutdown = true;
    let _ = snapshot_tx.send(state.snapshot());
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
            input_amount in 1u64..=1_000_000,
            spend_amount in 1u64..=1_000_000,
            sender in 0usize..16,
            receiver in 0usize..16,
        ) {
            prop_assume!(spend_amount <= input_amount);
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
            input_amount in 1u64..=500_000,
            spend_amount in 1u64..=500_000,
        ) {
            prop_assume!(spend_amount <= input_amount);
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
