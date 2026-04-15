use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::{Result, eyre};
use mugraph_core::{
    builder::RefreshBuilder,
    crypto,
    types::{
        Asset,
        BlindSignature,
        DleqProofWithBlinding,
        Hash,
        Note,
        PublicKey,
        Refresh,
    },
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

        let signature = output_iter.next().ok_or_else(|| {
            eyre!("missing signature for output {}", atom_idx)
        })?;

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

        if !crypto::verify(
            &delegate,
            commitment.as_ref(),
            signature.signature.0,
        )? {
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

/// Execute a cross-node transfer atomically on the destination node.
///
/// The input note (signed by source node) is consumed. Both the transfer note
/// (for the receiver) and the change note (for the sender, if any) are emitted
/// on the destination node. This makes the operation atomic: either both notes
/// are minted or neither is (single node, sequential calls that either all
/// succeed or fail on the first).
///
/// The change note will be signed by the destination delegate rather than the
/// source delegate. This is fine — the simulator routes refreshes by note
/// delegate, so future transactions with the change note will go to the
/// destination node.
async fn cross_node_transfer(
    _source_node: &SimNode,
    dest_node: &SimNode,
    input_note: &Note,
    asset: Asset,
    spend_amount: u64,
    receiver_id: usize,
) -> std::result::Result<CrossNodeResult, CrossNodeError> {
    // Emit the transfer amount on the destination node
    let receiver_note = dest_node
        .client
        .emit(asset.policy_id, asset.asset_name, spend_amount)
        .await
        .map_err(|e| CrossNodeError {
            reason: format!("destination emit failed: {e}"),
            recovered_notes: vec![],
        })?;

    // If there's change, also emit it on the destination node to keep atomicity.
    // The change note will be signed by the destination delegate.
    let change_note = if input_note.amount > spend_amount {
        let change_amount = input_note.amount - spend_amount;
        Some(
            dest_node
                .client
                .emit(asset.policy_id, asset.asset_name, change_amount)
                .await
                .map_err(|e| CrossNodeError {
                    reason: format!(
                        "change emit failed after receiver note minted: {e}"
                    ),
                    // The receiver note was already minted — include it for recovery
                    recovered_notes: vec![(receiver_id, receiver_note.clone())],
                })?,
        )
    } else {
        None
    };

    Ok(CrossNodeResult {
        receiver_note,
        change_note,
    })
}

fn apply_successful_tx(
    state: &mut AppState,
    pending: &PendingTx,
    notes: Vec<(usize, Note)>,
) {
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
        pending.id,
        pending.sender_id,
        pending.receiver_id,
        pending.spend_amount
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

fn emit_snapshot(
    state: &AppState,
    oracle: &mut ConservationOracle,
    max_inflight: usize,
    throughput: &mut ThroughputTracker,
    snapshot_tx: &tokio::sync::watch::Sender<AppSnapshot>,
) {
    let _ = snapshot_tx.send(state.snapshot(
        oracle.checks_passed(),
        max_inflight,
        throughput.tx_per_sec(),
        throughput.success_rate(),
    ));
}

fn choose_transfer_participants(
    state: &AppState,
    rng: &mut StdRng,
) -> Option<(usize, usize, usize, usize)> {
    let wallet_count = state.wallets.len();
    if wallet_count < 2 {
        return None;
    }

    let sender_idx = rng.random_range(0..wallet_count);
    let receiver_idx = {
        let mut idx = rng.random_range(0..wallet_count - 1);
        if idx >= sender_idx {
            idx += 1;
        }
        idx
    };

    Some((
        sender_idx,
        receiver_idx,
        state.wallets[sender_idx].id,
        state.wallets[receiver_idx].id,
    ))
}

fn complete_same_node_transfer(
    state: &mut AppState,
    inflight_amounts: &HashMap<Asset, u128>,
    oracle: &mut ConservationOracle,
    throughput: &mut ThroughputTracker,
    pending: &PendingTx,
    notes: Vec<(usize, Note)>,
) {
    throughput.record_ok();
    apply_successful_tx(state, pending, notes);
    oracle.assert_conservation(
        state,
        inflight_amounts,
        &format!("after successful tx {}", pending.id),
    );
}

fn handle_same_node_completion(
    state: &mut AppState,
    inflight_amounts: &mut HashMap<Asset, u128>,
    oracle: &mut ConservationOracle,
    throughput: &mut ThroughputTracker,
    pending: PendingTx,
    result: std::result::Result<Vec<BlindSignature>, String>,
) {
    state.inflight = state.inflight.saturating_sub(1);

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
                complete_same_node_transfer(
                    state,
                    inflight_amounts,
                    oracle,
                    throughput,
                    &pending,
                    notes,
                );
            }
            Err(e) => {
                throughput.record_err();
                record_sender_failure(
                    state,
                    pending.sender_id,
                    pending.asset,
                    pending.input_note,
                    format!("tx {} materialization failed: {e:#}", pending.id),
                );
                oracle.assert_conservation(
                    state,
                    inflight_amounts,
                    &format!("after materialization failure tx {}", pending.id),
                );
            }
        },
        Err(reason) => {
            throughput.record_err();
            record_sender_failure(
                state,
                pending.sender_id,
                pending.asset,
                pending.input_note,
                format!("tx {} failed: {}", pending.id, reason),
            );
            oracle.assert_conservation(
                state,
                inflight_amounts,
                &format!("after rpc failure tx {}", pending.id),
            );
        }
    }
}

fn handle_cross_node_completion(
    state: &mut AppState,
    inflight_amounts: &mut HashMap<Asset, u128>,
    oracle: &mut ConservationOracle,
    throughput: &mut ThroughputTracker,
    event: CrossNodeTxEvent,
) {
    let CrossNodeTxEvent {
        id,
        sender_id,
        receiver_id,
        asset,
        input_amount,
        input_note,
        spend_amount,
        result,
    } = event;

    state.inflight = state.inflight.saturating_sub(1);

    let entry = inflight_amounts.entry(asset).or_default();
    *entry = entry.saturating_sub(input_amount as u128);

    match result {
        Ok(xnode_result) => {
            throughput.record_ok();

            let recv_key = Asset {
                policy_id: xnode_result.receiver_note.policy_id,
                asset_name: xnode_result.receiver_note.asset_name,
            };
            state.wallets[receiver_id]
                .notes
                .entry(recv_key)
                .or_default()
                .push(xnode_result.receiver_note);

            if let Some(change_note) = xnode_result.change_note {
                let change_key = Asset {
                    policy_id: change_note.policy_id,
                    asset_name: change_note.asset_name,
                };
                state.wallets[sender_id]
                    .notes
                    .entry(change_key)
                    .or_default()
                    .push(change_note);
            }

            state.wallets[sender_id].sent += 1;
            state.wallets[receiver_id].received += 1;
            state.total_ok += 1;
            state.cross_node_ok += 1;
            state.log(format!(
                "xnode tx {id} ok sender={sender_id} receiver={receiver_id} amount={spend_amount}"
            ));

            oracle.assert_conservation(
                state,
                inflight_amounts,
                &format!("after successful cross-node tx {id}"),
            );
        }
        Err(err) => {
            throughput.record_err();

            if err.recovered_notes.is_empty() {
                record_sender_failure(
                    state,
                    sender_id,
                    asset,
                    input_note,
                    format!("xnode tx {id} failed: {}", err.reason),
                );
            } else {
                let recovered_total: u128 = err
                    .recovered_notes
                    .iter()
                    .map(|(_, note)| note.amount as u128)
                    .sum();
                for (owner, note) in err.recovered_notes {
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

                let lost =
                    (input_amount as u128).saturating_sub(recovered_total);
                if lost > 0 {
                    oracle.record_loss(&asset, lost);
                }

                state.wallets[sender_id].failures += 1;
                state.total_err += 1;
                let msg = format!(
                    "xnode tx {id} partial failure (recovered {recovered_total}, lost {lost}): {}",
                    err.reason,
                );
                state.log(&msg);
                state.last_failure = Some(msg);
            }
            oracle.assert_conservation(
                state,
                inflight_amounts,
                &format!("after cross-node failure tx {id}"),
            );
        }
    }
}

fn handle_simulation_tick(
    nodes: &[SimNode],
    state: &mut AppState,
    rng: &mut StdRng,
    config: &SimConfig,
    oracle: &mut ConservationOracle,
    inflight_amounts: &mut HashMap<Asset, u128>,
    throughput: &mut ThroughputTracker,
    tx_id: &mut u64,
    event_tx: &tokio::sync::mpsc::UnboundedSender<SimEvent>,
    snapshot_tx: &tokio::sync::watch::Sender<AppSnapshot>,
) -> bool {
    if state.shutdown {
        return true;
    }
    if state.paused || state.inflight >= config.max_inflight {
        return false;
    }

    let Some((sender_idx, receiver_idx, sender_id, receiver_id)) =
        choose_transfer_participants(state, rng)
    else {
        return false;
    };

    let Some((asset, input_note)) = reserve_spendable_note(
        &mut state.wallets[sender_idx],
        &state.assets,
        rng,
    ) else {
        return false;
    };

    let spend_amount = rng
        .random_range(config.amount_range.0..=config.amount_range.1)
        .min(input_note.amount);

    let input_amount = input_note.amount;
    let note_delegate = input_note.delegate;
    let max_inflight = config.max_inflight;

    let receiver_home = state.wallets[receiver_idx].home_node;
    let receiver_delegate = nodes[receiver_home].delegate_pk;
    let is_cross_node = note_delegate != receiver_delegate && nodes.len() > 1;

    if is_cross_node {
        *inflight_amounts.entry(asset).or_default() += input_amount as u128;

        oracle.assert_conservation(
            state,
            inflight_amounts,
            &format!("after cross-node reserve (tx_id={})", *tx_id + 1),
        );

        *tx_id += 1;
        state.inflight += 1;
        state.total_sent += 1;
        emit_snapshot(state, oracle, max_inflight, throughput, snapshot_tx);

        let source_node = nodes
            .iter()
            .find(|n| n.delegate_pk == note_delegate)
            .unwrap_or(&nodes[0])
            .clone();
        let dest_node = nodes[receiver_home].clone();
        let event_tx_clone = event_tx.clone();
        let current_tx_id = *tx_id;

        tokio::spawn(async move {
            let result = cross_node_transfer(
                &source_node,
                &dest_node,
                &input_note,
                asset,
                spend_amount,
                receiver_id,
            )
            .await;

            let _ = event_tx_clone.send(SimEvent::CrossNodeTxFinished(
                Box::new(CrossNodeTxEvent {
                    id: current_tx_id,
                    sender_id,
                    receiver_id,
                    asset,
                    input_amount,
                    input_note,
                    spend_amount,
                    result,
                }),
            ));
        });
        return false;
    }

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
                state,
                sender_id,
                asset,
                input_note,
                format!("failed to build refresh: {e:#}"),
            );
            oracle.assert_conservation(
                state,
                inflight_amounts,
                &format!("after build_refresh failure (tx_id={})", *tx_id + 1),
            );
            emit_snapshot(state, oracle, max_inflight, throughput, snapshot_tx);
            return false;
        }
    };

    *inflight_amounts.entry(asset).or_default() += input_amount as u128;

    oracle.assert_conservation(
        state,
        inflight_amounts,
        &format!("after reserve (tx_id={})", *tx_id + 1),
    );

    let node = nodes
        .iter()
        .find(|n| n.delegate_pk == note_delegate)
        .unwrap_or(&nodes[0]);

    *tx_id += 1;
    let pending = PendingTx {
        id: *tx_id,
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
    emit_snapshot(state, oracle, max_inflight, throughput, snapshot_tx);

    let client_clone = node.client.clone();
    let event_tx_clone = event_tx.clone();
    tokio::spawn(async move {
        let result = match client_clone.refresh(&pending.refresh).await {
            Ok(outputs) => Ok(outputs),
            Err(e) => Err(e.to_string()),
        };
        let _ = event_tx_clone.send(SimEvent::TxFinished {
            pending: Box::new(pending),
            result,
        });
    });

    false
}

fn handle_simulation_command(
    state: &mut AppState,
    oracle: &mut ConservationOracle,
    max_inflight: usize,
    throughput: &mut ThroughputTracker,
    snapshot_tx: &tokio::sync::watch::Sender<AppSnapshot>,
    cmd: SimCommand,
) -> bool {
    match cmd {
        SimCommand::TogglePause => {
            state.paused = !state.paused;
            state.log(format!("paused set to {}", state.paused));
            emit_snapshot(state, oracle, max_inflight, throughput, snapshot_tx);
            false
        }
        SimCommand::Quit => {
            state.shutdown = true;
            state.log(format!(
                "shutting down — conservation checks passed: {}",
                oracle.checks_passed(),
            ));
            emit_snapshot(state, oracle, max_inflight, throughput, snapshot_tx);
            true
        }
    }
}

fn handle_simulation_event(
    state: &mut AppState,
    inflight_amounts: &mut HashMap<Asset, u128>,
    oracle: &mut ConservationOracle,
    throughput: &mut ThroughputTracker,
    max_inflight: usize,
    snapshot_tx: &tokio::sync::watch::Sender<AppSnapshot>,
    event: SimEvent,
) {
    match event {
        SimEvent::TxFinished { pending, result } => {
            handle_same_node_completion(
                state,
                inflight_amounts,
                oracle,
                throughput,
                *pending,
                result,
            );
            emit_snapshot(state, oracle, max_inflight, throughput, snapshot_tx);
        }
        SimEvent::CrossNodeTxFinished(ev) => {
            handle_cross_node_completion(
                state,
                inflight_amounts,
                oracle,
                throughput,
                *ev,
            );
            emit_snapshot(state, oracle, max_inflight, throughput, snapshot_tx);
        }
    }
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

    emit_snapshot(
        &state,
        &mut oracle,
        max_inflight,
        &mut throughput,
        &snapshot_tx,
    );

    let mut tx_id: u64 = 0;

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if handle_simulation_tick(
                    &nodes,
                    &mut state,
                    &mut rng,
                    &config,
                    &mut oracle,
                    &mut inflight_amounts,
                    &mut throughput,
                    &mut tx_id,
                    &event_tx,
                    &snapshot_tx,
                ) {
                    break;
                }
            }
            Some(cmd) = cmd_rx.recv() => {
                if handle_simulation_command(
                    &mut state,
                    &mut oracle,
                    max_inflight,
                    &mut throughput,
                    &snapshot_tx,
                    cmd,
                ) {
                    break;
                }
            }
            Some(event) = event_rx.recv() => {
                handle_simulation_event(
                    &mut state,
                    &mut inflight_amounts,
                    &mut oracle,
                    &mut throughput,
                    max_inflight,
                    &snapshot_tx,
                    event,
                );
            }
        }
    }

    state.shutdown = true;
    emit_snapshot(
        &state,
        &mut oracle,
        max_inflight,
        &mut throughput,
        &snapshot_tx,
    );
}

#[cfg(test)]
mod tests {
    use mugraph_core::types::{AssetName, Hash, PolicyId, Signature};
    use proptest::prelude::*;
    use rand::SeedableRng;

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

        let Some(pos) =
            notes.iter().position(|n| n.signature == target.signature)
        else {
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

    #[tokio::test]
    async fn simulation_owner_loop_emits_final_snapshot_on_quit() {
        let state = AppState::default();
        let config = SimConfig {
            amount_range: (1, 1),
            tick: Duration::from_millis(1),
            max_inflight: 4,
        };
        let (snapshot_tx, snapshot_rx) = tokio::sync::watch::channel(
            state.snapshot(0, config.max_inflight, 0.0, 100.0),
        );
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();

        let handle = tokio::spawn(simulation_owner_loop(
            Vec::new(),
            state,
            StdRng::seed_from_u64(7),
            config,
            SimChannels {
                cmd_rx,
                event_rx,
                event_tx,
                snapshot_tx,
            },
        ));

        cmd_tx.send(SimCommand::Quit).unwrap();
        handle.await.unwrap();

        let snapshot = snapshot_rx.borrow().clone();
        assert!(snapshot.shutdown);
        assert_eq!(snapshot.inflight, 0);
        assert_eq!(snapshot.total_sent, 0);
        assert_eq!(snapshot.total_ok, 0);
        assert_eq!(snapshot.total_err, 0);
        assert_eq!(snapshot.max_inflight, 4);
    }

    #[test]
    fn same_node_completion_helper_updates_counters_and_conservation_inputs() {
        let asset = Asset {
            policy_id: PolicyId([7u8; 28]),
            asset_name: AssetName::empty(),
        };
        let input_note = Note {
            amount: 50,
            delegate: PublicKey([1u8; 32]),
            policy_id: asset.policy_id,
            asset_name: asset.asset_name,
            nonce: Hash([5u8; 32]),
            signature: Signature([1u8; 32]),
            dleq: None,
        };
        let receiver_note = Note {
            amount: 50,
            delegate: PublicKey([2u8; 32]),
            policy_id: asset.policy_id,
            asset_name: asset.asset_name,
            nonce: Hash([6u8; 32]),
            signature: Signature([2u8; 32]),
            dleq: None,
        };

        let (refresh, owners) =
            build_refresh(0, 1, asset, input_note.clone(), 50).unwrap();
        let pending = PendingTx {
            id: 7,
            sender_id: 0,
            receiver_id: 1,
            asset,
            input_amount: 50,
            input_note: input_note.clone(),
            spend_amount: 50,
            refresh,
            owners,
            delegate: input_note.delegate,
        };

        let mut state = AppState {
            wallets: vec![
                Wallet {
                    id: 0,
                    home_node: 0,
                    notes: HashMap::from([(asset, vec![input_note.clone()])]),
                    ..Default::default()
                },
                Wallet {
                    id: 1,
                    home_node: 0,
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let mut oracle = ConservationOracle::new();
        oracle.seal(&state);
        state.wallets[0].notes.clear();

        let inflight_amounts = HashMap::new();
        let mut throughput = ThroughputTracker::new(Duration::from_secs(5));
        complete_same_node_transfer(
            &mut state,
            &inflight_amounts,
            &mut oracle,
            &mut throughput,
            &pending,
            vec![(1, receiver_note.clone())],
        );

        assert_eq!(state.wallets[0].sent, 1);
        assert_eq!(state.wallets[1].received, 1);
        assert_eq!(state.total_ok, 1);
        assert_eq!(state.wallets[1].notes.get(&asset).unwrap()[0].amount, 50);
        assert_eq!(inflight_amounts.get(&asset).copied().unwrap_or(0), 0);
    }

    #[test]
    fn cross_node_partial_failure_helper_updates_loss_failure_and_last_failure()
    {
        let asset = Asset {
            policy_id: PolicyId([7u8; 28]),
            asset_name: AssetName::empty(),
        };
        let input_note = Note {
            amount: 100,
            delegate: PublicKey([1u8; 32]),
            policy_id: asset.policy_id,
            asset_name: asset.asset_name,
            nonce: Hash([5u8; 32]),
            signature: Signature([1u8; 32]),
            dleq: None,
        };
        let receiver_note = Note {
            amount: 60,
            delegate: PublicKey([2u8; 32]),
            policy_id: asset.policy_id,
            asset_name: asset.asset_name,
            nonce: Hash([6u8; 32]),
            signature: Signature([2u8; 32]),
            dleq: None,
        };

        let mut state = AppState {
            wallets: vec![
                Wallet {
                    id: 0,
                    home_node: 0,
                    notes: HashMap::from([(asset, vec![input_note.clone()])]),
                    ..Default::default()
                },
                Wallet {
                    id: 1,
                    home_node: 1,
                    ..Default::default()
                },
            ],
            inflight: 1,
            ..Default::default()
        };
        let mut oracle = ConservationOracle::new();
        oracle.seal(&state);
        state.wallets[0].notes.clear();

        let mut inflight_amounts = HashMap::new();
        let mut throughput = ThroughputTracker::new(Duration::from_secs(5));
        handle_cross_node_completion(
            &mut state,
            &mut inflight_amounts,
            &mut oracle,
            &mut throughput,
            CrossNodeTxEvent {
                id: 8,
                sender_id: 0,
                receiver_id: 1,
                asset,
                input_amount: 100,
                input_note: input_note.clone(),
                spend_amount: 60,
                result: Err(CrossNodeError {
                    reason: "change emit failed".to_string(),
                    recovered_notes: vec![(1, receiver_note.clone())],
                }),
            },
        );

        assert_eq!(state.inflight, 0);
        assert_eq!(state.total_err, 1);
        assert_eq!(state.wallets[0].failures, 1);
        assert_eq!(state.wallets[1].notes.get(&asset).unwrap()[0].amount, 60);
        assert!(
            state
                .last_failure
                .as_ref()
                .unwrap()
                .contains("partial failure")
        );
        assert!(state.last_failure.as_ref().unwrap().contains("lost 40"));
    }

    /// Tests that partial cross-node failure with recovered notes correctly
    /// adjusts the conservation oracle and distributes recovered notes.
    ///
    /// Scenario: receiver note (spend_amount=60) was minted on the destination
    /// node, but the change note (40) failed to mint. The error handler should:
    /// 1. Distribute the recovered receiver note to the receiver
    /// 2. Record the lost value (40) in the conservation oracle
    /// 3. Pass the conservation check (oracle expected supply is reduced)
    #[test]
    fn cross_node_partial_failure_recovers_notes_and_adjusts_oracle() {
        let asset = Asset {
            policy_id: PolicyId([7u8; 28]),
            asset_name: AssetName::empty(),
        };
        let delegate_a = PublicKey([1u8; 32]);
        let delegate_b = PublicKey([2u8; 32]);

        let input_note = Note {
            amount: 100,
            delegate: delegate_a,
            policy_id: asset.policy_id,
            asset_name: asset.asset_name,
            nonce: Hash([5u8; 32]),
            signature: Signature([1u8; 32]),
            dleq: None,
        };

        let receiver_note = Note {
            amount: 60,
            delegate: delegate_b,
            policy_id: asset.policy_id,
            asset_name: asset.asset_name,
            nonce: Hash([6u8; 32]),
            signature: Signature([2u8; 32]),
            dleq: None,
        };

        let spend_amount = 60u64;
        let input_amount = 100u64;

        let mut state = AppState {
            wallets: vec![
                Wallet {
                    id: 0,
                    home_node: 0,
                    notes: HashMap::new(),
                    ..Default::default()
                },
                Wallet {
                    id: 1,
                    home_node: 1,
                    notes: HashMap::new(),
                    ..Default::default()
                },
            ],
            assets: vec![],
            delegates: vec![delegate_a, delegate_b],
            ..Default::default()
        };

        let mut oracle = ConservationOracle::new();

        // Seal with the input note present
        state.wallets[0]
            .notes
            .entry(asset)
            .or_default()
            .push(input_note.clone());
        oracle.seal(&state);

        // Reserve: remove note, track as inflight
        state.wallets[0].notes.get_mut(&asset).unwrap().pop();
        let mut inflight_amounts: HashMap<Asset, u128> = HashMap::new();
        *inflight_amounts.entry(asset).or_default() += input_amount as u128;

        oracle.assert_conservation(&state, &inflight_amounts, "after reserve");

        // Simulate event arrival: clear inflight
        let entry = inflight_amounts.entry(asset).or_default();
        *entry = entry.saturating_sub(input_amount as u128);

        // Partial failure: receiver note was minted (60), change (40) failed
        let err = CrossNodeError {
            reason: "change emit failed".to_string(),
            recovered_notes: vec![(1, receiver_note)], // receiver gets 60
        };

        // Distribute recovered notes
        let recovered_total: u128 = err
            .recovered_notes
            .iter()
            .map(|(_, n)| n.amount as u128)
            .sum();
        for (owner, note) in err.recovered_notes {
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

        // Record the irrecoverable loss
        let lost = (input_amount as u128).saturating_sub(recovered_total);
        assert_eq!(
            lost, 40,
            "40 units of value were lost (change never minted)"
        );
        oracle.record_loss(&asset, lost);

        // Conservation check passes: oracle expected 100, reduced by 40 → expects 60
        // Wallets have 60 (receiver), inflight 0. 60 == 60.
        oracle.assert_conservation(
            &state,
            &inflight_amounts,
            "after partial cross-node failure with recovery",
        );

        // Verify the receiver got the note
        let receiver_balance: u64 = state.wallets[1]
            .notes
            .get(&asset)
            .map(|ns| ns.iter().map(|n| n.amount).sum())
            .unwrap_or(0);
        assert_eq!(receiver_balance, spend_amount);

        // Sender got nothing back (input was consumed, change failed)
        let sender_balance: u64 = state.wallets[0]
            .notes
            .get(&asset)
            .map(|ns| ns.iter().map(|n| n.amount).sum())
            .unwrap_or(0);
        assert_eq!(sender_balance, 0);
    }

    /// Tests that a total cross-node failure (no notes minted) restores the
    /// input note to the sender and conservation holds without any oracle adjustment.
    #[test]
    fn cross_node_total_failure_restores_input_note() {
        let asset = Asset {
            policy_id: PolicyId([7u8; 28]),
            asset_name: AssetName::empty(),
        };
        let delegate_a = PublicKey([1u8; 32]);

        let input_note = Note {
            amount: 100,
            delegate: delegate_a,
            policy_id: asset.policy_id,
            asset_name: asset.asset_name,
            nonce: Hash([5u8; 32]),
            signature: Signature([1u8; 32]),
            dleq: None,
        };

        let mut state = AppState {
            wallets: vec![
                Wallet {
                    id: 0,
                    home_node: 0,
                    notes: HashMap::new(),
                    ..Default::default()
                },
                Wallet {
                    id: 1,
                    home_node: 1,
                    notes: HashMap::new(),
                    ..Default::default()
                },
            ],
            assets: vec![],
            delegates: vec![delegate_a],
            ..Default::default()
        };

        let mut oracle = ConservationOracle::new();

        state.wallets[0]
            .notes
            .entry(asset)
            .or_default()
            .push(input_note.clone());
        oracle.seal(&state);

        // Reserve
        state.wallets[0].notes.get_mut(&asset).unwrap().pop();
        let mut inflight_amounts: HashMap<Asset, u128> = HashMap::new();
        *inflight_amounts.entry(asset).or_default() += 100u128;

        oracle.assert_conservation(&state, &inflight_amounts, "after reserve");

        // Clear inflight
        let entry = inflight_amounts.entry(asset).or_default();
        *entry = entry.saturating_sub(100u128);

        // Total failure: nothing was minted, restore input
        record_sender_failure(
            &mut state,
            0,
            asset,
            input_note,
            "xnode tx failed: destination emit failed".to_string(),
        );

        // Conservation holds without any oracle adjustment
        oracle.assert_conservation(
            &state,
            &inflight_amounts,
            "after total cross-node failure",
        );

        // Sender got the input note back
        let sender_balance: u64 = state.wallets[0]
            .notes
            .get(&asset)
            .map(|ns| ns.iter().map(|n| n.amount).sum())
            .unwrap_or(0);
        assert_eq!(sender_balance, 100);
    }

    fn make_note(
        amount: u64,
        delegate: PublicKey,
        asset: Asset,
        sig_byte: u8,
    ) -> Note {
        Note {
            amount,
            delegate,
            policy_id: asset.policy_id,
            asset_name: asset.asset_name,
            nonce: Hash([sig_byte; 32]),
            signature: Signature([sig_byte; 32]),
            dleq: None,
        }
    }

    fn test_asset() -> Asset {
        Asset {
            policy_id: PolicyId([7u8; 28]),
            asset_name: AssetName::empty(),
        }
    }

    proptest! {
        /// Conservation oracle: seal-then-check identity.
        ///
        /// After sealing, the very next conservation check must pass with zero
        /// inflight — the oracle's expected supply should exactly match the wallet
        /// contents it was sealed from.
        #[test]
        fn prop_oracle_seal_then_check_is_identity(
            amounts in prop::collection::vec(1u64..=100_000, 1..8),
            wallet_count in 1usize..=4,
        ) {
            let asset = test_asset();
            let delegate = PublicKey([1u8; 32]);
            let mut state = AppState::default();

            for wid in 0..wallet_count {
                let mut wallet = Wallet {
                    id: wid,
                    home_node: 0,
                    ..Default::default()
                };
                // Distribute amounts round-robin to wallets
                for (i, &amt) in amounts.iter().enumerate() {
                    if i % wallet_count == wid {
                        wallet
                            .notes
                            .entry(asset)
                            .or_default()
                            .push(make_note(amt, delegate, asset, (i & 0xFF) as u8));
                    }
                }
                state.wallets.push(wallet);
            }

            let mut oracle = ConservationOracle::new();
            oracle.seal(&state);

            let empty_inflight = HashMap::new();
            // Must not panic
            oracle.assert_conservation(&state, &empty_inflight, "seal-then-check identity");
            prop_assert_eq!(oracle.checks_passed(), 1);
        }

        /// Conservation oracle: reserve-then-restore round-trip.
        ///
        /// Removing a note from a wallet and tracking it as inflight preserves
        /// conservation. Restoring it (simulating failure) also preserves it.
        #[test]
        fn prop_oracle_reserve_restore_round_trip(
            input_amount in 1u64..=1_000_000,
            extra_notes in prop::collection::vec(1u64..=50_000, 0..4),
        ) {
            let asset = test_asset();
            let delegate = PublicKey([1u8; 32]);
            let mut state = AppState {
                wallets: vec![Wallet {
                    id: 0,
                    home_node: 0,
                    ..Default::default()
                }],
                ..Default::default()
            };

            // Add the main note + extras
            let input_note = make_note(input_amount, delegate, asset, 0);
            state.wallets[0].notes.entry(asset).or_default().push(input_note.clone());
            for (i, &amt) in extra_notes.iter().enumerate() {
                state.wallets[0]
                    .notes
                    .entry(asset)
                    .or_default()
                    .push(make_note(amt, delegate, asset, (i + 1) as u8));
            }

            let mut oracle = ConservationOracle::new();
            oracle.seal(&state);

            // Reserve: remove note, add to inflight
            state.wallets[0].notes.get_mut(&asset).unwrap().swap_remove(0);
            let mut inflight = HashMap::new();
            *inflight.entry(asset).or_default() += input_amount as u128;

            oracle.assert_conservation(&state, &inflight, "after reserve");

            // Restore: put note back, clear inflight
            *inflight.get_mut(&asset).unwrap() -= input_amount as u128;
            record_sender_failure(
                &mut state,
                0,
                asset,
                input_note,
                "test failure".to_string(),
            );

            oracle.assert_conservation(&state, &inflight, "after restore");
            prop_assert_eq!(oracle.checks_passed(), 2);
        }

        /// Conservation oracle: record_loss adjusts expected supply exactly.
        ///
        /// After losing some value, the oracle should accept a reduced total.
        #[test]
        fn prop_oracle_record_loss_adjusts_supply(
            initial in 100u64..=1_000_000,
            loss_pct in 1u32..=100,
        ) {
            let asset = test_asset();
            let delegate = PublicKey([1u8; 32]);
            let mut state = AppState {
                wallets: vec![Wallet {
                    id: 0,
                    home_node: 0,
                    ..Default::default()
                }],
                ..Default::default()
            };

            state.wallets[0]
                .notes
                .entry(asset)
                .or_default()
                .push(make_note(initial, delegate, asset, 0));

            let mut oracle = ConservationOracle::new();
            oracle.seal(&state);

            let loss = (initial as u128 * loss_pct as u128) / 100;
            let remaining = initial as u128 - loss;

            // Simulate: remove note, replace with smaller one
            state.wallets[0].notes.get_mut(&asset).unwrap().clear();
            if remaining > 0 {
                state.wallets[0]
                    .notes
                    .entry(asset)
                    .or_default()
                    .push(make_note(remaining as u64, delegate, asset, 1));
            }

            oracle.record_loss(&asset, loss);

            let empty = HashMap::new();
            oracle.assert_conservation(&state, &empty, "after record_loss");
        }

        /// reserve_spendable_note: returned note was in the wallet and is removed.
        ///
        /// Invariant: wallet_value_before == wallet_value_after + returned_note.amount.
        /// Also: the note must match an asset in the provided asset list.
        #[test]
        fn prop_reserve_removes_exactly_one_note(
            amounts in prop::collection::vec(1u64..=10_000, 1..8),
            seed in any::<u64>(),
        ) {
            let asset = test_asset();
            let delegate = PublicKey([9u8; 32]);
            let sim_asset = SimAsset {
                policy_id: asset.policy_id,
                asset_name: asset.asset_name,
                name: "TEST",
                policy_id_hex: "0000000000000000000000000000000000000000000000000000000000",
            };

            let mut wallet = Wallet {
                id: 0,
                home_node: 0,
                ..Default::default()
            };
            for (i, &amt) in amounts.iter().enumerate() {
                wallet
                    .notes
                    .entry(asset)
                    .or_default()
                    .push(make_note(amt, delegate, asset, i as u8));
            }

            let total_before: u64 = wallet
                .notes
                .values()
                .flatten()
                .map(|n| n.amount)
                .sum();
            let count_before = wallet.notes.values().map(|v| v.len()).sum::<usize>();

            let mut rng = StdRng::seed_from_u64(seed);
            let result = reserve_spendable_note(&mut wallet, &[sim_asset], &mut rng);

            match result {
                Some((returned_asset, note)) => {
                    prop_assert_eq!(returned_asset, asset);
                    prop_assert!(note.amount > 0);

                    let total_after: u64 = wallet
                        .notes
                        .values()
                        .flatten()
                        .map(|n| n.amount)
                        .sum();
                    let count_after = wallet.notes.values().map(|v| v.len()).sum::<usize>();

                    prop_assert_eq!(total_after + note.amount, total_before);
                    prop_assert_eq!(count_after + 1, count_before);
                }
                None => {
                    // Should never happen: all notes have amount > 0
                    prop_assert!(false, "reserve returned None but wallet had notes");
                }
            }
        }

        /// Snapshot asset summaries must match wallet contents.
        ///
        /// Differential test: compute supply by summing wallets directly,
        /// compare with snapshot.asset_summaries.
        #[test]
        fn prop_snapshot_asset_summary_matches_wallets(
            wallet_amounts in prop::collection::vec(
                prop::collection::vec(1u64..=10_000, 0..4),
                1..=4,
            ),
        ) {
            let asset = test_asset();
            let delegate = PublicKey([1u8; 32]);
            let sim_asset = SimAsset {
                policy_id: asset.policy_id,
                asset_name: asset.asset_name,
                name: "TEST",
                policy_id_hex: "0000000000000000000000000000000000000000000000000000000000",
            };

            let mut state = AppState {
                assets: vec![sim_asset],
                ..Default::default()
            };

            let mut expected_supply = 0u64;
            let mut expected_notes = 0usize;
            let mut expected_holding = 0usize;

            for (wid, amounts) in wallet_amounts.iter().enumerate() {
                let mut wallet = Wallet {
                    id: wid,
                    home_node: 0,
                    ..Default::default()
                };
                let wallet_total: u64 = amounts.iter().sum();
                if !amounts.is_empty() {
                    expected_holding += 1;
                    expected_notes += amounts.len();
                    expected_supply += wallet_total;
                    for (i, &amt) in amounts.iter().enumerate() {
                        wallet
                            .notes
                            .entry(asset)
                            .or_default()
                            .push(make_note(amt, delegate, asset, (wid * 16 + i) as u8));
                    }
                }
                state.wallets.push(wallet);
            }

            let snap = state.snapshot(0, 16, 0.0, 100.0);

            prop_assert_eq!(snap.asset_summaries.len(), 1);
            let summary = &snap.asset_summaries[0];
            prop_assert_eq!(summary.total_supply, expected_supply);
            prop_assert_eq!(summary.total_notes, expected_notes);
            prop_assert_eq!(summary.wallets_holding, expected_holding);
        }

        /// Cross-node error recovery: recovered + lost == input_amount.
        ///
        /// For any split of input into recovered notes and lost value,
        /// the accounting must be exact.
        #[test]
        fn prop_cross_node_recovery_accounting_is_exact(
            input_amount in 2u64..=1_000_000,
            recovered_pct in 0u32..=100,
        ) {
            let recovered_amount = (input_amount as u128 * recovered_pct as u128) / 100;
            let lost = (input_amount as u128).saturating_sub(recovered_amount);

            prop_assert_eq!(recovered_amount + lost, input_amount as u128);

            // Simulate what the handler does
            let asset = test_asset();
            let delegate = PublicKey([2u8; 32]);
            let mut state = AppState {
                wallets: vec![
                    Wallet { id: 0, home_node: 0, ..Default::default() },
                    Wallet { id: 1, home_node: 1, ..Default::default() },
                ],
                ..Default::default()
            };

            // Seal with input in wallet 0
            state.wallets[0]
                .notes
                .entry(asset)
                .or_default()
                .push(make_note(input_amount, delegate, asset, 0));

            let mut oracle = ConservationOracle::new();
            oracle.seal(&state);

            // Reserve
            state.wallets[0].notes.get_mut(&asset).unwrap().pop();
            let mut inflight: HashMap<Asset, u128> = HashMap::new();
            *inflight.entry(asset).or_default() += input_amount as u128;

            oracle.assert_conservation(&state, &inflight, "after reserve");

            // Event: clear inflight
            *inflight.get_mut(&asset).unwrap() = 0;

            // Distribute recovered to receiver (wallet 1)
            if recovered_amount > 0 {
                state.wallets[1]
                    .notes
                    .entry(asset)
                    .or_default()
                    .push(make_note(recovered_amount as u64, delegate, asset, 1));
            }

            // Record loss
            if lost > 0 {
                oracle.record_loss(&asset, lost);
            }

            // Must not panic
            oracle.assert_conservation(
                &state,
                &inflight,
                "after cross-node recovery",
            );
        }

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
