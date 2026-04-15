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
    pub network: String,
    pub notes: Vec<StoredNote>,
    pub delegate_pk: Option<PublicKey>,
    pub cardano_script_address: Option<String>,
    pub has_orphaned_blinding_factors: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResult {
    pub envelope: String,
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
    let notes = state
        .store
        .list_notes(&network)
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

    Ok(WalletSnapshot {
        network,
        notes,
        delegate_pk,
        cardano_script_address: script_addr,
        has_orphaned_blinding_factors: !orphans.is_empty(),
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

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let envelope = serde_json::json!({
        "network": input.network,
        "delegate_pk": format!("{delegate_pk}"),
        "sender_label": label,
        "created_at": now,
        "notes": notes,
    });

    // Mark notes as spent
    for note in &notes {
        state
            .store
            .update_note_status(&input.network, &note.nonce, NoteStatus::Spent)
            .map_err(|e| e.to_string())?;
    }

    Ok(SendResult {
        envelope: serde_json::to_string(&envelope)
            .map_err(|e| e.to_string())?,
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

    Ok(SyncResult {
        node_reachable: true,
        delegate_pk_changed: pk_changed,
    })
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
            network: "preprod".to_string(),
            notes: vec![],
            delegate_pk: None,
            cardano_script_address: None,
            has_orphaned_blinding_factors: false,
        };
        let json = serde_json::to_string(&snap).unwrap();
        assert!(json.contains("preprod"));
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
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("envelope"));
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
