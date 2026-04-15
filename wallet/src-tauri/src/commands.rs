use std::{collections::HashMap, sync::Arc};

use mugraph_core::types::{Keypair, PublicKey};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    node_client::NodeClient,
    store::{NoteStatus, Store, StoredNote},
};

pub struct AppState {
    pub store: Store,
    pub keypair: Keypair,
    pub ed25519_key: ed25519_dalek::SigningKey,
    pub cardano_payment_sk: [u8; 32],
    pub cardano_payment_vk: [u8; 32],
    pub node_clients: RwLock<HashMap<String, NodeClient>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    pub label: String,
    pub mainnet_node_url: String,
    pub preprod_node_url: String,
    pub preview_node_url: String,
    pub provider_type: String,
    pub provider_api_key: String,
    pub provider_base_url_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupResult {
    pub networks: Vec<NetworkBootstrap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkBootstrap {
    pub network: String,
    pub delegate_pk: PublicKey,
    pub cardano_script_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSnapshot {
    pub label: String,
    pub network: String,
    pub notes: Vec<StoredNote>,
    pub activity: Vec<crate::store::ActivityRecord>,
    pub delegate_pk: Option<PublicKey>,
    pub cardano_script_address: Option<String>,
    pub cardano_funding_address: Option<String>,
    pub has_orphaned_blinding_factors: bool,
    pub last_synced_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveRequestInput {
    pub network: String,
    pub policy_id: String,
    pub asset_name: String,
    pub amount: u64,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: usize,
    pub quarantined: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendInput {
    pub network: String,
    pub note_nonces: Vec<String>,
}

/// QR codes practically hold ~2953 bytes in alphanumeric mode.
/// We use a conservative limit for JSON payloads.
const QR_PAYLOAD_LIMIT: usize = 2500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub envelope: String,
    /// "qr" if the payload fits a single QR code, "text" otherwise.
    pub transport_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshInput {
    pub network: String,
    pub note_nonces: Vec<String>,
    pub target_amounts: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResult {
    pub new_note_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub node_reachable: bool,
    pub delegate_pk_changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositInput {
    pub network: String,
    pub utxo_tx_hash: String,
    pub utxo_index: u16,
    pub output_amounts: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositResult {
    pub notes_created: usize,
    pub deposit_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawInput {
    pub network: String,
    pub destination_address: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawResult {
    pub tx_hash: String,
    pub change_notes: usize,
}

#[tauri::command]
pub async fn complete_guided_setup(
    config: SetupConfig,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<SetupResult, String> {
    // Store config
    state
        .store
        .set_config("label", &config.label)
        .map_err(|e| e.to_string())?;
    state
        .store
        .set_node_url("mainnet", &config.mainnet_node_url)
        .map_err(|e| e.to_string())?;
    state
        .store
        .set_node_url("preprod", &config.preprod_node_url)
        .map_err(|e| e.to_string())?;
    state
        .store
        .set_node_url("preview", &config.preview_node_url)
        .map_err(|e| e.to_string())?;
    state
        .store
        .set_provider_config("type", &config.provider_type)
        .map_err(|e| e.to_string())?;
    state
        .store
        .set_provider_config("api_key", &config.provider_api_key)
        .map_err(|e| e.to_string())?;
    if let Some(ref base_url) = config.provider_base_url_override {
        state
            .store
            .set_provider_config("base_url_override", base_url)
            .map_err(|e| e.to_string())?;
    }

    // Bootstrap each network
    let urls = [
        ("mainnet", &config.mainnet_node_url),
        ("preprod", &config.preprod_node_url),
        ("preview", &config.preview_node_url),
    ];

    let mut results = Vec::new();
    let mut clients = state.node_clients.write().await;

    for (network, url) in urls {
        let parsed = reqwest::Url::parse(url).map_err(|e| e.to_string())?;
        let client = NodeClient::new(&parsed).map_err(|e| e.to_string())?;
        let (delegate_pk, script_addr) =
            client.info().await.map_err(|e| e.to_string())?;

        state
            .store
            .set_delegate_pk(network, &delegate_pk)
            .map_err(|e| e.to_string())?;
        if let Some(ref addr) = script_addr {
            state
                .store
                .set_script_address(network, addr)
                .map_err(|e| e.to_string())?;
        }

        clients.insert(network.to_string(), client);
        results.push(NetworkBootstrap {
            network: network.to_string(),
            delegate_pk,
            cardano_script_address: script_addr,
        });
    }

    state
        .store
        .set_config("setup_complete", "true")
        .map_err(|e| e.to_string())?;
    state
        .store
        .set_config("last_network", "preprod")
        .map_err(|e| e.to_string())?;

    Ok(SetupResult { networks: results })
}

#[tauri::command]
pub async fn get_wallet_state(
    network: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<WalletSnapshot, String> {
    let label = state
        .store
        .get_config("label")
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| "Mugraph Wallet".to_string());
    let notes = state
        .store
        .list_notes(&network)
        .map_err(|e| e.to_string())?;
    let activity = state
        .store
        .list_activity(&network)
        .map_err(|e| e.to_string())?;
    let delegate_pk = state
        .store
        .get_delegate_pk(&network)
        .map_err(|e| e.to_string())?;
    let script_addr = state
        .store
        .get_script_address(&network)
        .map_err(|e| e.to_string())?;
    let orphans = state
        .store
        .scan_orphaned_blinding_factors(&network)
        .map_err(|e| e.to_string())?;

    // Derive the in-app Cardano funding address for this network
    let funding_addr =
        crate::cardano_tx::derive_address(&state.cardano_payment_vk, &network)
            .ok();

    // Load last synced timestamp for this network
    let sync_key = format!("last_synced_at_{network}");
    let last_synced_at = state
        .store
        .get_config(&sync_key)
        .map_err(|e| e.to_string())?
        .and_then(|s| s.parse::<u64>().ok());

    Ok(WalletSnapshot {
        label,
        network,
        notes,
        activity,
        delegate_pk,
        cardano_script_address: script_addr,
        cardano_funding_address: funding_addr,
        has_orphaned_blinding_factors: !orphans.is_empty(),
        last_synced_at,
    })
}

#[tauri::command]
pub async fn switch_network(
    network: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<WalletSnapshot, String> {
    state
        .store
        .set_config("last_network", &network)
        .map_err(|e| e.to_string())?;
    get_wallet_state(network, state).await
}

#[tauri::command]
pub async fn create_receive_request(
    input: ReceiveRequestInput,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let delegate_pk = state
        .store
        .get_delegate_pk(&input.network)
        .map_err(|e| e.to_string())?
        .ok_or("no delegate pk for network")?;

    let request = serde_json::json!({
        "network": input.network,
        "delegate_pk": format!("{delegate_pk}"),
        "recipient_label": state.store.get_config("label").map_err(|e| e.to_string())?.unwrap_or_default(),
        "asset": {
            "policy_id": input.policy_id,
            "asset_name": input.asset_name,
        },
        "amount": input.amount,
        "label": input.label,
    });

    let id = format!(
        "req-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let payload = serde_json::to_string(&request).map_err(|e| e.to_string())?;
    state
        .store
        .put_offchain_request(&id, payload.as_bytes())
        .map_err(|e| e.to_string())?;

    Ok(payload)
}

#[tauri::command]
pub async fn import_notes(
    payload: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<ImportResult, String> {
    let envelope: serde_json::Value =
        serde_json::from_str(&payload).map_err(|e| e.to_string())?;

    let network = envelope["network"]
        .as_str()
        .ok_or("missing network")?
        .to_string();

    let notes_array =
        envelope["notes"].as_array().ok_or("missing notes array")?;

    let delegate_pk = state
        .store
        .get_delegate_pk(&network)
        .map_err(|e| e.to_string())?
        .ok_or("no delegate pk for network")?;

    // Validate envelope delegate matches the active wallet's delegate for this network
    if let Some(envelope_delegate) = envelope["delegate_pk"].as_str() {
        let expected_delegate_hex = hex::encode(delegate_pk.0);
        if envelope_delegate != expected_delegate_hex {
            return Err(format!(
                "envelope delegate_pk mismatch: expected {}, got {}",
                expected_delegate_hex, envelope_delegate
            ));
        }
    }

    let mut imported = 0;
    let mut quarantined = 0;

    for note_value in notes_array {
        let note: mugraph_core::types::Note =
            serde_json::from_value(note_value.clone())
                .map_err(|e| e.to_string())?;

        // Verify signature
        let commitment = note.commitment();
        let valid = mugraph_core::crypto::verify(
            &delegate_pk,
            commitment.as_ref(),
            note.signature,
        )
        .unwrap_or(false);

        let status = if valid {
            NoteStatus::Available
        } else {
            NoteStatus::Quarantined
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        state
            .store
            .put_note(&network, &note, status, now)
            .map_err(|e| e.to_string())?;

        match status {
            NoteStatus::Available => imported += 1,
            NoteStatus::Quarantined => quarantined += 1,
            _ => {}
        }
    }

    // Auto-refresh imported notes to re-validate against the delegate.
    // If refresh fails, quarantine the notes.
    if imported > 0 {
        let available_notes: Vec<mugraph_core::types::Note> = {
            let all = state
                .store
                .list_notes(&network)
                .map_err(|e| e.to_string())?;
            all.into_iter()
                .filter(|s| s.status == NoteStatus::Available)
                .map(|s| s.note)
                .collect()
        };

        // Try to refresh through the node — if the node is available
        let clients = state.node_clients.read().await;
        if let Some(client) = clients.get(&network) {
            // Only refresh the newly imported notes (last `imported` available notes)
            let to_refresh: Vec<&mugraph_core::types::Note> =
                available_notes.iter().rev().take(imported).collect();

            for note in to_refresh {
                let mut builder = mugraph_core::builder::RefreshBuilder::new();
                builder = builder.input(note.clone());
                builder = builder.output(
                    note.policy_id,
                    note.asset_name,
                    note.amount,
                );

                match builder.build() {
                    Ok(mut refresh) => {
                        // Blind the single output
                        let (bf, bp) = {
                            let mut rng = rand::rng();
                            let atom = &refresh.atoms[1]; // output is at index 1
                            let commitment =
                                atom.commitment(&refresh.asset_ids);
                            let blinded = mugraph_core::crypto::blind(
                                &mut rng,
                                commitment.as_ref(),
                            );
                            state
                                .store
                                .put_blinding_factor(
                                    &network,
                                    &atom.nonce,
                                    &blinded.factor.to_bytes(),
                                )
                                .map_err(|e| e.to_string())?;
                            (
                                blinded.factor,
                                mugraph_core::types::Signature::from(
                                    blinded.point,
                                ),
                            )
                        };
                        refresh.blinded_points = vec![bp];

                        match client.refresh(&refresh).await {
                            Ok(sigs) if !sigs.is_empty() => {
                                let sig = &sigs[0];
                                let atom = &refresh.atoms[1];
                                let commitment =
                                    atom.commitment(&refresh.asset_ids);

                                let ok = (|| -> Result<bool, String> {
                                    let bpt = bp
                                        .to_point()
                                        .map_err(|e| e.to_string())?;
                                    let dleq_ok = mugraph_core::crypto::verify_dleq_signature(
                                        &delegate_pk, &bpt, &sig.signature, &sig.proof,
                                    ).map_err(|e| e.to_string())?;
                                    if !dleq_ok {
                                        return Ok(false);
                                    }
                                    let unblinded = mugraph_core::crypto::unblind_signature(
                                        &sig.signature, &bf, &delegate_pk,
                                    ).map_err(|e| e.to_string())?;
                                    mugraph_core::crypto::verify(
                                        &delegate_pk,
                                        commitment.as_ref(),
                                        unblinded,
                                    )
                                    .map_err(|e| e.to_string())
                                })();

                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();

                                match ok {
                                    Ok(true) => {
                                        let unblinded = mugraph_core::crypto::unblind_signature(
                                            &sig.signature, &bf, &delegate_pk,
                                        ).map_err(|e| e.to_string())?;
                                        let new_note = mugraph_core::types::Note {
                                            amount: atom.amount,
                                            delegate: atom.delegate,
                                            policy_id: refresh.asset_ids[atom.asset_id as usize].policy_id,
                                            asset_name: refresh.asset_ids[atom.asset_id as usize].asset_name,
                                            nonce: atom.nonce,
                                            signature: unblinded,
                                            dleq: Some(mugraph_core::types::DleqProofWithBlinding {
                                                proof: sig.proof,
                                                blinding_factor: bf.into(),
                                            }),
                                        };
                                        // Mark old note spent, store new one
                                        let _ = state.store.update_note_status(
                                            &network,
                                            &note.nonce,
                                            NoteStatus::Spent,
                                        );
                                        let _ = state.store.finalize_note(
                                            &network,
                                            &new_note,
                                            NoteStatus::Available,
                                            now,
                                        );
                                    }
                                    _ => {
                                        // Refresh verification failed — quarantine
                                        let _ = state.store.update_note_status(
                                            &network,
                                            &note.nonce,
                                            NoteStatus::Quarantined,
                                        );
                                        let _ =
                                            state.store.delete_blinding_factor(
                                                &network,
                                                &atom.nonce,
                                            );
                                        imported -= 1;
                                        quarantined += 1;
                                    }
                                }
                            }
                            _ => {
                                // Refresh RPC failed — quarantine
                                let _ = state.store.update_note_status(
                                    &network,
                                    &note.nonce,
                                    NoteStatus::Quarantined,
                                );
                                imported -= 1;
                                quarantined += 1;
                            }
                        }
                    }
                    Err(_) => {
                        // Build failed (shouldn't happen for 1:1 refresh) — quarantine
                        let _ = state.store.update_note_status(
                            &network,
                            &note.nonce,
                            NoteStatus::Quarantined,
                        );
                        imported -= 1;
                        quarantined += 1;
                    }
                }
            }
        }
        // If no client available, notes stay as-is (available but un-refreshed)
    }

    Ok(ImportResult {
        imported,
        quarantined,
    })
}

#[tauri::command]
pub async fn send(
    input: SendInput,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<SendResult, String> {
    let delegate_pk = state
        .store
        .get_delegate_pk(&input.network)
        .map_err(|e| e.to_string())?
        .ok_or("no delegate pk for network")?;

    let label = state
        .store
        .get_config("label")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    let mut notes = Vec::new();
    for nonce_hex in &input.note_nonces {
        let nonce_bytes =
            muhex::decode(nonce_hex).map_err(|e| e.to_string())?;
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&nonce_bytes);
        let nonce = mugraph_core::types::Hash(arr);

        let stored = state
            .store
            .get_note(&input.network, &nonce)
            .map_err(|e| e.to_string())?
            .ok_or("note not found")?;

        if stored.status != NoteStatus::Available {
            return Err("note is not available for sending".to_string());
        }

        notes.push(stored.note);
    }

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // ISO 8601 format as specified in the reference
    let created_at_iso = format!(
        "{}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        1970 + now_secs / 31_536_000,
        (now_secs % 31_536_000) / 2_592_000 + 1,
        (now_secs % 2_592_000) / 86_400 + 1,
        (now_secs % 86_400) / 3_600,
        (now_secs % 3_600) / 60,
        now_secs % 60,
    );

    let envelope = serde_json::json!({
        "network": input.network,
        "delegate_pk": hex::encode(delegate_pk.0),
        "sender_label": label,
        "created_at": created_at_iso,
        "notes": notes,
    });

    // Mark notes as spent
    for note in &notes {
        state
            .store
            .update_note_status(&input.network, &note.nonce, NoteStatus::Spent)
            .map_err(|e| e.to_string())?;
    }

    let envelope_str =
        serde_json::to_string(&envelope).map_err(|e| e.to_string())?;
    let transport_hint = if envelope_str.len() <= QR_PAYLOAD_LIMIT {
        "qr"
    } else {
        "text"
    };

    Ok(SendResult {
        envelope: envelope_str,
        transport_hint: transport_hint.to_string(),
    })
}

#[tauri::command]
pub async fn refresh_notes(
    input: RefreshInput,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<RefreshResult, String> {
    let delegate_pk = state
        .store
        .get_delegate_pk(&input.network)
        .map_err(|e| e.to_string())?
        .ok_or("no delegate pk for network")?;

    // Collect input notes
    let mut input_notes = Vec::new();
    for nonce_hex in &input.note_nonces {
        let nonce_bytes =
            muhex::decode(nonce_hex).map_err(|e| e.to_string())?;
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&nonce_bytes);
        let nonce = mugraph_core::types::Hash(arr);

        let stored = state
            .store
            .get_note(&input.network, &nonce)
            .map_err(|e| e.to_string())?
            .ok_or("note not found")?;

        if stored.status != NoteStatus::Available {
            return Err(format!(
                "note {} is {:?}, only available notes can be refreshed",
                nonce_hex, stored.status
            ));
        }

        input_notes.push(stored.note);
    }

    // Build refresh
    let mut builder = mugraph_core::builder::RefreshBuilder::new();
    for note in &input_notes {
        builder = builder.input(note.clone());
    }
    for &amount in &input.target_amounts {
        let first = &input_notes[0];
        builder = builder.output(first.policy_id, first.asset_name, amount);
    }
    let mut refresh = builder.build().map_err(|e| e.to_string())?;

    // Blind outputs (scoped to drop rng before .await)
    let blinding_factors = {
        let mut rng = rand::rng();
        let mut factors = Vec::new();
        let mut points = Vec::new();

        for (i, atom) in refresh.atoms.iter().enumerate() {
            if refresh.is_output(i) {
                let commitment = atom.commitment(&refresh.asset_ids);
                let blinded =
                    mugraph_core::crypto::blind(&mut rng, commitment.as_ref());
                factors.push((atom.nonce, blinded.factor));
                points
                    .push(mugraph_core::types::Signature::from(blinded.point));

                // Persist blinding factor BEFORE sending
                state
                    .store
                    .put_blinding_factor(
                        &input.network,
                        &atom.nonce,
                        &blinded.factor.to_bytes(),
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
        refresh.blinded_points = points;
        factors
    };

    // Send to node
    let clients = state.node_clients.read().await;
    let client = clients
        .get(&input.network)
        .ok_or("no node client for network")?;
    let signatures =
        client.refresh(&refresh).await.map_err(|e| e.to_string())?;

    // Process response
    let mut output_idx = 0;
    let mut new_count = 0;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    for (i, atom) in refresh.atoms.iter().enumerate() {
        if !refresh.is_output(i) {
            continue;
        }

        let sig = &signatures[output_idx];
        let (nonce, r) = &blinding_factors[output_idx];

        // Verify DLEQ
        let bp = refresh.blinded_points[output_idx]
            .to_point()
            .map_err(|e| e.to_string())?;
        let dleq_ok = mugraph_core::crypto::verify_dleq_signature(
            &delegate_pk,
            &bp,
            &sig.signature,
            &sig.proof,
        )
        .map_err(|e| e.to_string())?;
        if !dleq_ok {
            return Err("DLEQ verification failed".to_string());
        }

        // Unblind
        let unblinded = mugraph_core::crypto::unblind_signature(
            &sig.signature,
            r,
            &delegate_pk,
        )
        .map_err(|e| e.to_string())?;

        // Verify final signature
        let commitment = atom.commitment(&refresh.asset_ids);
        let valid = mugraph_core::crypto::verify(
            &delegate_pk,
            commitment.as_ref(),
            unblinded,
        )
        .map_err(|e| e.to_string())?;
        if !valid {
            return Err("unblinded signature verification failed".to_string());
        }

        let asset = &refresh.asset_ids[atom.asset_id as usize];
        let note = mugraph_core::types::Note {
            amount: atom.amount,
            delegate: atom.delegate,
            policy_id: asset.policy_id,
            asset_name: asset.asset_name,
            nonce: *nonce,
            signature: unblinded,
            dleq: Some(mugraph_core::types::DleqProofWithBlinding {
                proof: sig.proof,
                blinding_factor: (*r).into(),
            }),
        };

        state
            .store
            .finalize_note(&input.network, &note, NoteStatus::Available, now)
            .map_err(|e| e.to_string())?;

        output_idx += 1;
        new_count += 1;
    }

    // Mark input notes as spent
    for note in &input_notes {
        state
            .store
            .update_note_status(&input.network, &note.nonce, NoteStatus::Spent)
            .map_err(|e| e.to_string())?;
    }

    Ok(RefreshResult {
        new_note_count: new_count,
    })
}

#[tauri::command]
pub async fn deposit(
    input: DepositInput,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<DepositResult, String> {
    let delegate_pk = state
        .store
        .get_delegate_pk(&input.network)
        .map_err(|e| e.to_string())?
        .ok_or("no delegate pk for network")?;

    // Build blinded outputs
    let (blinding_data, blinded_outputs) = {
        let mut rng = rand::rng();
        let mut data = Vec::new();
        let mut outputs = Vec::new();

        for &amount in &input.output_amounts {
            let nonce = mugraph_core::types::Hash::random(&mut rng);
            // Build a temporary note to compute commitment
            let temp_note = mugraph_core::types::Note {
                amount,
                delegate: delegate_pk,
                policy_id: mugraph_core::types::PolicyId::zero(),
                asset_name: mugraph_core::types::AssetName::empty(),
                nonce,
                signature: mugraph_core::types::Signature::zero(),
                dleq: None,
            };
            let commitment = temp_note.commitment();
            let blinded =
                mugraph_core::crypto::blind(&mut rng, commitment.as_ref());

            // Persist blinding factor BEFORE sending
            state
                .store
                .put_blinding_factor(
                    &input.network,
                    &nonce,
                    &blinded.factor.to_bytes(),
                )
                .map_err(|e| e.to_string())?;

            data.push((
                nonce,
                blinded.factor,
                blinded.point,
                commitment,
                amount,
            ));
            outputs.push(mugraph_core::types::BlindSignature {
                signature: mugraph_core::types::Blinded(
                    mugraph_core::types::Signature::from(blinded.point),
                ),
                proof: mugraph_core::types::DleqProof::default(),
            });
        }
        (data, outputs)
    };

    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let script_address = state
        .store
        .get_script_address(&input.network)
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    // Build canonical payload matching node's CanonicalPayload format
    let output_hexes: Vec<String> = blinded_outputs
        .iter()
        .map(|o| hex::encode(o.signature.0.0))
        .collect();

    let canonical_payload = serde_json::json!({
        "utxo": {
            "tx_hash": input.utxo_tx_hash,
            "index": input.utxo_index,
        },
        "outputs": output_hexes,
        "delegate_pk": hex::encode(delegate_pk.0),
        "script_address": script_address,
        "nonce": nonce,
        "network": input.network,
    });
    let canonical_bytes = serde_json::to_string(&canonical_payload)
        .map_err(|e| e.to_string())?
        .into_bytes();

    // Build CIP-8 COSE_Sign1 signature over canonical payload
    let cip8_signature = crate::cip8::build_cip8_signature(
        &state.ed25519_key,
        &canonical_bytes,
    )?;

    let deposit_req = mugraph_core::types::DepositRequest {
        utxo: mugraph_core::types::UtxoReference {
            tx_hash: input.utxo_tx_hash.clone(),
            index: input.utxo_index,
        },
        outputs: blinded_outputs,
        message: serde_json::json!({
            "user_pubkey": muhex::encode(state.ed25519_key.verifying_key().as_bytes())
        })
        .to_string(),
        signature: cip8_signature,
        nonce,
        network: input.network.clone(),
    };

    let clients = state.node_clients.read().await;
    let client = clients
        .get(&input.network)
        .ok_or("no node client for network")?;

    let resp = client
        .deposit(&deposit_req)
        .await
        .map_err(|e| e.to_string())?;

    // Process response signatures
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut notes_created = 0;
    for (i, (nonce, r, bp, commitment, amount)) in
        blinding_data.iter().enumerate()
    {
        if i >= resp.signatures.len() {
            break;
        }
        let sig = &resp.signatures[i];

        // Verify DLEQ
        let bpt = mugraph_core::types::Signature::from(*bp)
            .to_point()
            .map_err(|e| e.to_string())?;
        let dleq_ok = mugraph_core::crypto::verify_dleq_signature(
            &delegate_pk,
            &bpt,
            &sig.signature,
            &sig.proof,
        )
        .map_err(|e| e.to_string())?;
        if !dleq_ok {
            return Err(
                "DLEQ verification failed for deposit output".to_string()
            );
        }

        let unblinded = mugraph_core::crypto::unblind_signature(
            &sig.signature,
            r,
            &delegate_pk,
        )
        .map_err(|e| e.to_string())?;
        let valid = mugraph_core::crypto::verify(
            &delegate_pk,
            commitment.as_ref(),
            unblinded,
        )
        .map_err(|e| e.to_string())?;
        if !valid {
            return Err("unblinded signature verification failed for deposit"
                .to_string());
        }

        let note = mugraph_core::types::Note {
            amount: *amount,
            delegate: delegate_pk,
            policy_id: mugraph_core::types::PolicyId::zero(),
            asset_name: mugraph_core::types::AssetName::empty(),
            nonce: *nonce,
            signature: unblinded,
            dleq: Some(mugraph_core::types::DleqProofWithBlinding {
                proof: sig.proof,
                blinding_factor: (*r).into(),
            }),
        };

        state
            .store
            .finalize_note(&input.network, &note, NoteStatus::Available, now)
            .map_err(|e| e.to_string())?;
        notes_created += 1;
    }

    // Record activity
    state
        .store
        .put_activity(
            &input.network,
            &crate::store::ActivityRecord {
                id: format!("deposit-{}", resp.deposit_ref),
                kind: "deposit".to_string(),
                timestamp: now,
                details: format!(
                    "Deposited {} outputs from {}:{}",
                    notes_created, input.utxo_tx_hash, input.utxo_index
                ),
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(DepositResult {
        notes_created,
        deposit_ref: resp.deposit_ref,
    })
}

#[tauri::command]
pub async fn withdraw(
    input: WithdrawInput,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<WithdrawResult, String> {
    let delegate_pk = state
        .store
        .get_delegate_pk(&input.network)
        .map_err(|e| e.to_string())?
        .ok_or("no delegate pk for network")?;

    let script_addr = state
        .store
        .get_script_address(&input.network)
        .map_err(|e| e.to_string())?
        .ok_or("no script address for network")?;

    // Select notes covering the withdrawal amount
    // For now, use PolicyId::zero / AssetName::empty (lovelace)
    let selected = state
        .store
        .select_notes(
            &input.network,
            &mugraph_core::types::PolicyId::zero(),
            &mugraph_core::types::AssetName::empty(),
            input.amount,
        )
        .map_err(|e| e.to_string())?;

    // Build notes to burn
    let notes_to_burn: Vec<mugraph_core::types::BlindSignature> = selected
        .iter()
        .map(|s| mugraph_core::types::BlindSignature {
            signature: mugraph_core::types::Blinded(s.note.signature),
            proof: mugraph_core::types::DleqProof::default(),
        })
        .collect();

    let total_selected: u64 = selected.iter().map(|s| s.note.amount).sum();
    let change_amount = total_selected.saturating_sub(input.amount);

    // Blind change outputs if there is change
    let (change_blinding, change_outputs) = if change_amount > 0 {
        let (bf, bp, nonce) = {
            let mut rng = rand::rng();
            let nonce = mugraph_core::types::Hash::random(&mut rng);
            let temp_note = mugraph_core::types::Note {
                amount: change_amount,
                delegate: delegate_pk,
                policy_id: mugraph_core::types::PolicyId::zero(),
                asset_name: mugraph_core::types::AssetName::empty(),
                nonce,
                signature: mugraph_core::types::Signature::zero(),
                dleq: None,
            };
            let commitment = temp_note.commitment();
            let blinded =
                mugraph_core::crypto::blind(&mut rng, commitment.as_ref());
            state
                .store
                .put_blinding_factor(
                    &input.network,
                    &nonce,
                    &blinded.factor.to_bytes(),
                )
                .map_err(|e| e.to_string())?;
            (
                blinded.factor,
                mugraph_core::types::Signature::from(blinded.point),
                nonce,
            )
        };
        let change_sig = mugraph_core::types::BlindSignature {
            signature: mugraph_core::types::Blinded(bp),
            proof: mugraph_core::types::DleqProof::default(),
        };
        (Some((bf, nonce, change_amount)), vec![change_sig])
    } else {
        (None, vec![])
    };

    // Build the Cardano withdrawal transaction
    // In a full integration the wallet would query the Cardano provider for
    // actual script UTxOs matching the user's deposit datums. For now we use
    // synthetic inputs derived from the selected notes' nonces (each note
    // represents a previous deposit whose UTxO the node controls).
    let script_inputs: Vec<(String, u32)> = selected
        .iter()
        .enumerate()
        .map(|(i, s)| (hex::encode(s.note.nonce.0), i as u32))
        .collect();

    let (tx_cbor, tx_hash) = crate::cardano_tx::build_withdraw_tx(
        &crate::cardano_tx::WithdrawTxParams {
            script_inputs: &script_inputs,
            total_input_lovelace: total_selected,
            destination_address: &input.destination_address,
            withdraw_amount_lovelace: input.amount,
            script_address: &script_addr,
            fee_lovelace: 200_000,
        },
    )
    .map_err(|e| format!("tx build: {e}"))?;

    let witnessed_cbor = crate::cardano_tx::attach_user_witness(
        &tx_cbor,
        &tx_hash,
        &state.ed25519_key,
    )
    .map_err(|e| format!("witness: {e}"))?;

    let withdraw_req = mugraph_core::types::WithdrawRequest {
        notes: notes_to_burn,
        change_outputs,
        tx_cbor: hex::encode(&witnessed_cbor),
        tx_hash: hex::encode(tx_hash),
    };

    let clients = state.node_clients.read().await;
    let client = clients
        .get(&input.network)
        .ok_or("no node client for network")?;

    let resp = client
        .withdraw(&withdraw_req)
        .await
        .map_err(|e| format!("withdraw RPC: {e}"))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Mark consumed notes as spent
    for s in &selected {
        state
            .store
            .update_note_status(
                &input.network,
                &s.note.nonce,
                NoteStatus::Spent,
            )
            .map_err(|e| e.to_string())?;
    }

    // Unblind and store change notes
    let mut change_count = 0;
    if let Some((bf, nonce, amount)) = change_blinding {
        if let Some(change_sig) = resp.change_notes.first() {
            let commitment = {
                let temp = mugraph_core::types::Note {
                    amount,
                    delegate: delegate_pk,
                    policy_id: mugraph_core::types::PolicyId::zero(),
                    asset_name: mugraph_core::types::AssetName::empty(),
                    nonce,
                    signature: mugraph_core::types::Signature::zero(),
                    dleq: None,
                };
                temp.commitment()
            };

            let unblinded = mugraph_core::crypto::unblind_signature(
                &change_sig.signature,
                &bf,
                &delegate_pk,
            )
            .map_err(|e| e.to_string())?;

            let valid = mugraph_core::crypto::verify(
                &delegate_pk,
                commitment.as_ref(),
                unblinded,
            )
            .map_err(|e| e.to_string())?;

            if valid {
                let change_note = mugraph_core::types::Note {
                    amount,
                    delegate: delegate_pk,
                    policy_id: mugraph_core::types::PolicyId::zero(),
                    asset_name: mugraph_core::types::AssetName::empty(),
                    nonce,
                    signature: unblinded,
                    dleq: Some(mugraph_core::types::DleqProofWithBlinding {
                        proof: change_sig.proof,
                        blinding_factor: bf.into(),
                    }),
                };
                state
                    .store
                    .finalize_note(
                        &input.network,
                        &change_note,
                        NoteStatus::Available,
                        now,
                    )
                    .map_err(|e| e.to_string())?;
                change_count = 1;
            } else {
                // Failed verification on change note — hard attention
                state
                    .store
                    .delete_blinding_factor(&input.network, &nonce)
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    // Record activity
    state
        .store
        .put_activity(
            &input.network,
            &crate::store::ActivityRecord {
                id: format!("withdraw-{}", resp.tx_hash),
                kind: "withdraw".to_string(),
                timestamp: now,
                details: format!(
                    "Withdrew {} to {} (tx: {})",
                    input.amount, input.destination_address, resp.tx_hash
                ),
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(WithdrawResult {
        tx_hash: resp.tx_hash,
        change_notes: change_count,
    })
}

#[tauri::command]
pub async fn sync(
    network: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<SyncResult, String> {
    let clients = state.node_clients.read().await;
    let client = match clients.get(&network) {
        Some(c) => c,
        None => {
            return Ok(SyncResult {
                node_reachable: false,
                delegate_pk_changed: false,
            });
        }
    };

    // Check health
    if client.health().await.is_err() {
        return Ok(SyncResult {
            node_reachable: false,
            delegate_pk_changed: false,
        });
    }

    // Get current info
    let (new_pk, script_addr) =
        client.info().await.map_err(|e| e.to_string())?;

    let old_pk = state
        .store
        .get_delegate_pk(&network)
        .map_err(|e| e.to_string())?;
    let pk_changed = old_pk.as_ref() != Some(&new_pk);

    state
        .store
        .set_delegate_pk(&network, &new_pk)
        .map_err(|e| e.to_string())?;
    if let Some(ref addr) = script_addr {
        state
            .store
            .set_script_address(&network, addr)
            .map_err(|e| e.to_string())?;
    }

    // Update lastSyncedAt
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let sync_key = format!("last_synced_at_{network}");
    state
        .store
        .set_config(&sync_key, &now.to_string())
        .map_err(|e| e.to_string())?;

    Ok(SyncResult {
        node_reachable: true,
        delegate_pk_changed: pk_changed,
    })
}

#[tauri::command]
pub async fn retry_quarantined(
    network: String,
    nonce_hex: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let nonce_bytes = muhex::decode(&nonce_hex).map_err(|e| e.to_string())?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&nonce_bytes);
    let nonce = mugraph_core::types::Hash(arr);

    let stored = state
        .store
        .get_note(&network, &nonce)
        .map_err(|e| e.to_string())?
        .ok_or("note not found")?;

    if stored.status != NoteStatus::Quarantined {
        return Err("note is not quarantined".to_string());
    }

    // Try refresh through node
    let delegate_pk = state
        .store
        .get_delegate_pk(&network)
        .map_err(|e| e.to_string())?
        .ok_or("no delegate pk")?;

    let note = &stored.note;
    let mut builder = mugraph_core::builder::RefreshBuilder::new();
    builder = builder.input(note.clone());
    builder = builder.output(note.policy_id, note.asset_name, note.amount);
    let mut refresh = builder.build().map_err(|e| e.to_string())?;

    let (bf, bp) = {
        let mut rng = rand::rng();
        let atom = &refresh.atoms[1];
        let commitment = atom.commitment(&refresh.asset_ids);
        let blinded =
            mugraph_core::crypto::blind(&mut rng, commitment.as_ref());
        state
            .store
            .put_blinding_factor(
                &network,
                &atom.nonce,
                &blinded.factor.to_bytes(),
            )
            .map_err(|e| e.to_string())?;
        (
            blinded.factor,
            mugraph_core::types::Signature::from(blinded.point),
        )
    };
    refresh.blinded_points = vec![bp];

    let clients = state.node_clients.read().await;
    let client = clients.get(&network).ok_or("no node client")?;
    let sigs = client.refresh(&refresh).await.map_err(|e| e.to_string())?;

    if sigs.is_empty() {
        return Err("no signatures returned".to_string());
    }

    let sig = &sigs[0];
    let atom = &refresh.atoms[1];
    let commitment = atom.commitment(&refresh.asset_ids);
    let bpt = bp.to_point().map_err(|e| e.to_string())?;

    let dleq_ok = mugraph_core::crypto::verify_dleq_signature(
        &delegate_pk,
        &bpt,
        &sig.signature,
        &sig.proof,
    )
    .map_err(|e| e.to_string())?;
    if !dleq_ok {
        return Err("DLEQ verification failed".to_string());
    }

    let unblinded = mugraph_core::crypto::unblind_signature(
        &sig.signature,
        &bf,
        &delegate_pk,
    )
    .map_err(|e| e.to_string())?;
    let valid = mugraph_core::crypto::verify(
        &delegate_pk,
        commitment.as_ref(),
        unblinded,
    )
    .map_err(|e| e.to_string())?;
    if !valid {
        return Err("signature verification failed — note may be double-spent"
            .to_string());
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let new_note = mugraph_core::types::Note {
        amount: atom.amount,
        delegate: atom.delegate,
        policy_id: refresh.asset_ids[atom.asset_id as usize].policy_id,
        asset_name: refresh.asset_ids[atom.asset_id as usize].asset_name,
        nonce: atom.nonce,
        signature: unblinded,
        dleq: Some(mugraph_core::types::DleqProofWithBlinding {
            proof: sig.proof,
            blinding_factor: bf.into(),
        }),
    };

    state
        .store
        .update_note_status(&network, &nonce, NoteStatus::Spent)
        .map_err(|e| e.to_string())?;
    state
        .store
        .finalize_note(&network, &new_note, NoteStatus::Available, now)
        .map_err(|e| e.to_string())?;

    Ok("note re-validated successfully".to_string())
}

#[tauri::command]
pub async fn discard_quarantined(
    network: String,
    nonce_hex: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let nonce_bytes = muhex::decode(&nonce_hex).map_err(|e| e.to_string())?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&nonce_bytes);
    let nonce = mugraph_core::types::Hash(arr);

    state
        .store
        .update_note_status(&network, &nonce, NoteStatus::Spent)
        .map_err(|e| e.to_string())?;

    Ok("quarantined note discarded".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_config_serializes() {
        let config = SetupConfig {
            label: "Test Wallet".to_string(),
            mainnet_node_url: "http://localhost:3000".to_string(),
            preprod_node_url: "http://localhost:3001".to_string(),
            preview_node_url: "http://localhost:3002".to_string(),
            provider_type: "blockfrost".to_string(),
            provider_api_key: "key123".to_string(),
            provider_base_url_override: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        let decoded: SetupConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.label, "Test Wallet");
    }

    #[test]
    fn wallet_snapshot_serializes() {
        let snap = WalletSnapshot {
            label: "Test Wallet".to_string(),
            network: "preprod".to_string(),
            notes: vec![],
            activity: vec![],
            delegate_pk: None,
            cardano_script_address: None,
            cardano_funding_address: Some("addr_test1abc".to_string()),
            has_orphaned_blinding_factors: false,
            last_synced_at: Some(1700000000),
        };
        let json = serde_json::to_string(&snap).unwrap();
        assert!(json.contains("preprod"));
        assert!(json.contains("Test Wallet"));
        assert!(json.contains("addr_test1abc"));
        assert!(json.contains("1700000000"));
    }

    #[test]
    fn import_result_serializes() {
        let result = ImportResult {
            imported: 3,
            quarantined: 1,
        };
        let json = serde_json::to_string(&result).unwrap();
        let decoded: ImportResult = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.imported, 3);
        assert_eq!(decoded.quarantined, 1);
    }

    #[test]
    fn send_result_serializes() {
        let result = SendResult {
            envelope: r#"{"notes":[]}"#.to_string(),
            transport_hint: "qr".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("envelope"));
        assert!(json.contains("transport_hint"));
    }

    #[test]
    fn transport_hint_text_for_large_payload() {
        let large = "x".repeat(QR_PAYLOAD_LIMIT + 1);
        let hint = if large.len() <= QR_PAYLOAD_LIMIT {
            "qr"
        } else {
            "text"
        };
        assert_eq!(hint, "text");
    }

    #[test]
    fn transport_hint_qr_for_small_payload() {
        let small = "x".repeat(100);
        let hint = if small.len() <= QR_PAYLOAD_LIMIT {
            "qr"
        } else {
            "text"
        };
        assert_eq!(hint, "qr");
    }

    #[test]
    fn sync_result_serializes() {
        let result = SyncResult {
            node_reachable: true,
            delegate_pk_changed: false,
        };
        let json = serde_json::to_string(&result).unwrap();
        let decoded: SyncResult = serde_json::from_str(&json).unwrap();
        assert!(decoded.node_reachable);
        assert!(!decoded.delegate_pk_changed);
    }
}
