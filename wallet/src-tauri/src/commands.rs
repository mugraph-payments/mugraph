use std::{collections::HashMap, sync::Arc};

use mugraph_core::types::{AssetName, Keypair, PolicyId, PublicKey};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    node_client::NodeClient,
    provider::CardanoProvider,
    store::{NoteStatus, Store, StoredNote},
};

/// Convert Unix timestamp to ISO 8601 string (UTC).
fn unix_to_iso8601(ts: u64) -> String {
    let secs_per_day: u64 = 86_400;
    let days = ts / secs_per_day;
    let time_of_day = ts % secs_per_day;
    let h = time_of_day / 3600;
    let m = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;

    // Days since 1970-01-01 to civil date (Algorithm from Howard Hinnant)
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };

    format!("{year}-{month:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

fn parse_asset(
    policy_id: &Option<String>,
    asset_name: &Option<String>,
) -> (PolicyId, AssetName) {
    let pid = policy_id
        .as_ref()
        .and_then(|s| {
            let bytes = hex::decode(s).ok()?;
            if bytes.len() != 28 {
                return None;
            }
            let mut arr = [0u8; 28];
            arr.copy_from_slice(&bytes);
            Some(PolicyId(arr))
        })
        .unwrap_or_else(PolicyId::zero);
    let aname = asset_name
        .as_ref()
        .and_then(|s| AssetName::new(s.as_bytes()).ok())
        .unwrap_or_else(AssetName::empty);
    (pid, aname)
}

pub struct AppState {
    pub store: Store,
    pub keypair: Keypair,
    pub ed25519_key: ed25519_dalek::SigningKey,
    pub cardano_payment_sk: [u8; 32],
    pub cardano_payment_vk: [u8; 32],
    pub node_clients: RwLock<HashMap<String, NodeClient>>,
    pub provider: RwLock<Option<CardanoProvider>>,
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
    /// Guided setup has been completed at least once. The frontend uses this
    /// to gate on showing the setup form before the main wallet shell.
    pub setup_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingUtxo {
    pub tx_hash: String,
    pub output_index: u16,
    pub lovelace: u64,
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
    #[serde(default)]
    pub policy_id: Option<String>,
    #[serde(default)]
    pub asset_name: Option<String>,
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
    #[serde(default)]
    pub policy_id: Option<String>,
    #[serde(default)]
    pub asset_name: Option<String>,
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
    complete_guided_setup_impl(config, state.inner().clone()).await
}

pub async fn complete_guided_setup_impl(
    config: SetupConfig,
    state: Arc<AppState>,
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
        let (delegate_pk, script_addr, payment_vk_hex) =
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
        if let Some(ref vk_hex) = payment_vk_hex {
            let vk_bytes = hex::decode(vk_hex)
                .map_err(|e| format!("invalid node payment_vk hex: {e}"))?;
            state
                .store
                .set_node_payment_vk(network, &vk_bytes)
                .map_err(|e| e.to_string())?;
        }

        clients.insert(network.to_string(), client);
        results.push(NetworkBootstrap {
            network: network.to_string(),
            delegate_pk,
            cardano_script_address: script_addr,
        });
    }

    // Initialize the Cardano provider with the configured credentials
    // One provider reused across all networks, with network-specific endpoint
    // derived from the provider type.
    {
        let base_override = config.provider_base_url_override.as_deref();
        // Create a provider for the default network; the same credentials work
        // across networks — only the base URL changes.
        let cardano_provider = CardanoProvider::new(
            &config.provider_type,
            &config.provider_api_key,
            "preprod",
            base_override,
        )
        .map_err(|e| e.to_string())?;
        *state.provider.write().await = Some(cardano_provider);
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
    get_wallet_state_impl(network, state.inner().clone()).await
}

pub async fn get_wallet_state_impl(
    network: String,
    state: Arc<AppState>,
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

    let setup_complete = state
        .store
        .get_config("setup_complete")
        .map_err(|e| e.to_string())?
        .as_deref()
        == Some("true");

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
        setup_complete,
    })
}

#[tauri::command]
pub async fn list_funding_utxos(
    network: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<FundingUtxo>, String> {
    list_funding_utxos_impl(network, state.inner().clone()).await
}

pub async fn list_funding_utxos_impl(
    network: String,
    state: Arc<AppState>,
) -> Result<Vec<FundingUtxo>, String> {
    let funding_address =
        crate::cardano_tx::derive_address(&state.cardano_payment_vk, &network)
            .map_err(|e| format!("derive funding address: {e}"))?;

    let provider_guard = state.provider.read().await;
    let provider = provider_guard
        .as_ref()
        .ok_or("no Cardano provider configured")?;
    let utxos = provider
        .get_address_utxos(&funding_address)
        .await
        .map_err(|e| e.to_string())?;

    Ok(utxos
        .into_iter()
        .filter_map(|u| {
            let lovelace = u
                .amount
                .iter()
                .find(|a| a.unit == "lovelace")
                .and_then(|a| a.quantity.parse::<u64>().ok())?;
            Some(FundingUtxo {
                tx_hash: u.tx_hash,
                output_index: u.output_index,
                lovelace,
            })
        })
        .collect())
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
    import_notes_impl(payload, state.inner().clone()).await
}

pub async fn import_notes_impl(
    payload: String,
    state: Arc<AppState>,
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

    let mut notes = Vec::with_capacity(notes_array.len());
    for note_value in notes_array {
        let note: mugraph_core::types::Note =
            serde_json::from_value(note_value.clone())
                .map_err(|e| e.to_string())?;
        notes.push(note);
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // The wallet needs the delegate's denomination keys to check a note.
    // Without the node, keep the notes in quarantine until a retry.
    let clients = state.node_clients.read().await;
    let Some(client) = clients.get(&network) else {
        for note in &notes {
            state
                .store
                .put_note(&network, note, NoteStatus::Quarantined, now)
                .map_err(|e| e.to_string())?;
        }
        return Ok(ImportResult {
            imported: 0,
            quarantined: notes.len(),
        });
    };

    let mut imported = 0;
    let mut quarantined = 0;
    let mut keysets: HashMap<mugraph_core::types::Asset, Vec<PublicKey>> =
        HashMap::new();

    for note in notes {
        let asset = mugraph_core::types::Asset {
            policy_id: note.policy_id,
            asset_name: note.asset_name,
        };
        if let std::collections::hash_map::Entry::Vacant(entry) =
            keysets.entry(asset)
        {
            entry.insert(crate::notes::fetch_keyset(client, &asset).await?);
        }

        if note.delegate != delegate_pk
            || !crate::notes::verify_received_note(&note, &keysets[&asset])
        {
            state
                .store
                .put_note(&network, &note, NoteStatus::Quarantined, now)
                .map_err(|e| e.to_string())?;
            quarantined += 1;
            continue;
        }

        state
            .store
            .put_note(&network, &note, NoteStatus::Available, now)
            .map_err(|e| e.to_string())?;

        // The sender still knows this note, so swap it for a new note that
        // only this wallet knows. If the node refuses, the sender (or
        // someone else) spent it first.
        match crate::notes::refresh_through_node(
            &state.store,
            &network,
            client,
            std::slice::from_ref(&note),
            &[(asset, note.amount)],
        )
        .await
        {
            Ok(new_notes) => {
                state
                    .store
                    .update_note_status(
                        &network,
                        &note.nonce,
                        NoteStatus::Spent,
                    )
                    .map_err(|e| e.to_string())?;
                for new_note in &new_notes {
                    state
                        .store
                        .finalize_note(
                            &network,
                            new_note,
                            NoteStatus::Available,
                            now,
                        )
                        .map_err(|e| e.to_string())?;
                }
                imported += 1;
            }
            Err(_) => {
                state
                    .store
                    .update_note_status(
                        &network,
                        &note.nonce,
                        NoteStatus::Quarantined,
                    )
                    .map_err(|e| e.to_string())?;
                quarantined += 1;
            }
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
    send_impl(input, state.inner().clone()).await
}

pub async fn send_impl(
    input: SendInput,
    state: Arc<AppState>,
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
    let created_at_iso = unix_to_iso8601(now_secs);

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

    // Record send activity
    let send_now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let total_sent: u64 = notes.iter().map(|n| n.amount).sum();
    state
        .store
        .put_activity(
            &input.network,
            &crate::store::ActivityRecord {
                id: format!("send-{send_now}"),
                kind: "send".to_string(),
                timestamp: send_now,
                details: format!(
                    "Sent {} notes totalling {}",
                    notes.len(),
                    total_sent
                ),
            },
        )
        .map_err(|e| e.to_string())?;

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

    let first = input_notes.first().ok_or("no input notes")?;
    let asset = mugraph_core::types::Asset {
        policy_id: first.policy_id,
        asset_name: first.asset_name,
    };
    let outputs: Vec<(mugraph_core::types::Asset, u64)> = input
        .target_amounts
        .iter()
        .map(|&amount| (asset, amount))
        .collect();

    let clients = state.node_clients.read().await;
    let client = clients
        .get(&input.network)
        .ok_or("no node client for network")?;
    let new_notes = crate::notes::refresh_through_node(
        &state.store,
        &input.network,
        client,
        &input_notes,
        &outputs,
    )
    .await?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    for note in &new_notes {
        state
            .store
            .finalize_note(&input.network, note, NoteStatus::Available, now)
            .map_err(|e| e.to_string())?;
    }
    let new_count = new_notes.len();

    // Mark input notes as spent
    for note in &input_notes {
        state
            .store
            .update_note_status(&input.network, &note.nonce, NoteStatus::Spent)
            .map_err(|e| e.to_string())?;
    }

    // Record refresh activity
    let refresh_now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let total_refreshed: u64 = input_notes.iter().map(|n| n.amount).sum();
    state
        .store
        .put_activity(
            &input.network,
            &crate::store::ActivityRecord {
                id: format!("refresh-{refresh_now}"),
                kind: "refresh".to_string(),
                timestamp: refresh_now,
                details: format!(
                    "Refreshed {} inputs into {} outputs totalling {}",
                    input_notes.len(),
                    new_count,
                    total_refreshed,
                ),
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(RefreshResult {
        new_note_count: new_count,
    })
}

#[tauri::command]
pub async fn deposit(
    input: DepositInput,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<DepositResult, String> {
    deposit_impl(input, state.inner().clone()).await
}

pub async fn deposit_impl(
    input: DepositInput,
    state: Arc<AppState>,
) -> Result<DepositResult, String> {
    let delegate_pk = state
        .store
        .get_delegate_pk(&input.network)
        .map_err(|e| e.to_string())?
        .ok_or("no delegate pk for network")?;

    let (deposit_policy_id, deposit_asset_name) =
        parse_asset(&input.policy_id, &input.asset_name);

    let script_address = state
        .store
        .get_script_address(&input.network)
        .map_err(|e| e.to_string())?
        .ok_or("no script address for network")?;

    let node_payment_vk = state
        .store
        .get_node_payment_vk(&input.network)
        .map_err(|e| e.to_string())?
        .ok_or("no node payment_vk; rerun guided setup or /sync")?;
    if node_payment_vk.len() != 32 {
        return Err(format!(
            "node payment_vk has unexpected length {} (want 32)",
            node_payment_vk.len()
        ));
    }
    let node_pubkey_hash = crate::cip8::blake2b_224(&node_payment_vk);

    let funding_address = crate::cardano_tx::derive_address(
        &state.cardano_payment_vk,
        &input.network,
    )
    .map_err(|e| format!("derive funding address: {e}"))?;

    // Look up the funding UTxO via the configured Cardano provider so we
    // know its lovelace value. We run this inside a scoped read so the lock
    // is released before any subsequent state mutation.
    let funding_lovelace = {
        let provider_guard = state.provider.read().await;
        let provider = provider_guard
            .as_ref()
            .ok_or("no Cardano provider configured")?;
        let utxos = provider
            .get_address_utxos(&funding_address)
            .await
            .map_err(|e| e.to_string())?;
        let funding = utxos
            .iter()
            .find(|u| {
                u.tx_hash == input.utxo_tx_hash
                    && u.output_index == input.utxo_index
            })
            .ok_or_else(|| {
                format!(
                    "funding UTxO {}:{} not found at {}",
                    input.utxo_tx_hash, input.utxo_index, funding_address
                )
            })?;
        funding
            .amount
            .iter()
            .find(|a| a.unit == "lovelace")
            .and_then(|a| a.quantity.parse::<u64>().ok())
            .ok_or("funding UTxO has no lovelace amount")?
    };

    let deposit_amount: u64 = input.output_amounts.iter().sum();
    let fee_lovelace: u64 = 200_000;
    if funding_lovelace < deposit_amount.saturating_add(fee_lovelace) {
        return Err(format!(
            "funding UTxO has {funding_lovelace} lovelace; need at least \
             {} (deposit + {fee_lovelace} fee)",
            deposit_amount + fee_lovelace
        ));
    }

    // Build blinded outputs (and persist their blinding factors before any
    // network call leaves the wallet). Each amount becomes one note for
    // each of its denominations.
    let deposit_asset = mugraph_core::types::Asset {
        policy_id: deposit_policy_id,
        asset_name: deposit_asset_name,
    };
    let requested: Vec<(mugraph_core::types::Asset, u64)> = input
        .output_amounts
        .iter()
        .map(|&amount| (deposit_asset, amount))
        .collect();
    let (blinded_outputs, pending_notes) = crate::notes::blind_new_notes(
        &state.store,
        &input.network,
        delegate_pk,
        &requested,
    )?;

    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // The same bytes that the node rebuilds. They do not include the
    // deposit UTxO ref: the datum's intent_hash must be in the deposit tx
    // body, so the UTxO's tx_hash is not known yet.
    let canonical_bytes = mugraph_core::types::deposit_intent_payload(
        &blinded_outputs,
        &delegate_pk,
        &script_address,
        nonce,
        &input.network,
    );

    // Build CIP-8 COSE_Sign1 signature over canonical payload.
    let cip8_signature = crate::cip8::build_cip8_signature(
        &state.ed25519_key,
        &canonical_bytes,
    )?;

    // Build the deposit Cardano tx (funding UTxO -> script address with
    // inline datum) and submit it via the provider. The deposit UTxO ends
    // up at index 0 of the resulting tx (build_deposit_tx places it first).
    let user_ed25519_vk: [u8; 32] =
        state.ed25519_key.verifying_key().to_bytes();
    let (tx_cbor, tx_hash_bytes) = crate::cardano_tx::build_deposit_tx(
        &crate::cardano_tx::DepositTxParams {
            input_tx_hash: &input.utxo_tx_hash,
            input_index: input.utxo_index as u32,
            input_amount_lovelace: funding_lovelace,
            deposit_amount_lovelace: deposit_amount,
            script_address_bech32: &script_address,
            user_ed25519_vk: &user_ed25519_vk,
            node_payment_vk: &node_pubkey_hash,
            canonical_payload: &canonical_bytes,
            change_address_bech32: &funding_address,
            fee_lovelace,
        },
    )
    .map_err(|e| format!("build deposit tx: {e}"))?;

    let witnessed_cbor = crate::cardano_tx::attach_user_witness(
        &tx_cbor,
        &tx_hash_bytes,
        &state.ed25519_key,
    )
    .map_err(|e| format!("witness deposit tx: {e}"))?;

    let submitted_tx_hash = {
        let provider_guard = state.provider.read().await;
        let provider = provider_guard
            .as_ref()
            .ok_or("no Cardano provider configured")?;
        provider
            .submit_tx(&witnessed_cbor)
            .await
            .map_err(|e| format!("submit deposit tx: {e}"))?
            .tx_hash
    };

    let expected_tx_hash = hex::encode(tx_hash_bytes);
    if submitted_tx_hash != expected_tx_hash {
        return Err(format!(
            "provider returned tx_hash {submitted_tx_hash}, expected \
             {expected_tx_hash} (CBOR mismatch)"
        ));
    }

    let deposit_req = mugraph_core::types::DepositRequest {
        utxo: mugraph_core::types::UtxoReference {
            tx_hash: expected_tx_hash,
            index: 0,
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

    let notes =
        crate::notes::finish_notes(client, pending_notes, &resp.signatures)
            .await?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    for note in &notes {
        state
            .store
            .finalize_note(&input.network, note, NoteStatus::Available, now)
            .map_err(|e| e.to_string())?;
    }
    let notes_created = notes.len();

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
    withdraw_impl(input, state.inner().clone()).await
}

pub async fn withdraw_impl(
    input: WithdrawInput,
    state: Arc<AppState>,
) -> Result<WithdrawResult, String> {
    const FEE_LOVELACE: u64 = 200_000;

    let delegate_pk = state
        .store
        .get_delegate_pk(&input.network)
        .map_err(|e| e.to_string())?
        .ok_or("no delegate pk for network")?;

    let (withdraw_policy_id, withdraw_asset_name) =
        parse_asset(&input.policy_id, &input.asset_name);
    let withdraw_asset = mugraph_core::types::Asset {
        policy_id: withdraw_policy_id,
        asset_name: withdraw_asset_name,
    };
    if !withdraw_asset.is_ada() {
        return Err("only ADA withdrawals are supported".to_string());
    }

    let script_addr = state
        .store
        .get_script_address(&input.network)
        .map_err(|e| e.to_string())?
        .ok_or("no script address for network")?;

    let node_payment_vk = state
        .store
        .get_node_payment_vk(&input.network)
        .map_err(|e| e.to_string())?
        .ok_or("no node payment_vk; rerun guided setup or /sync")?;
    let node_pubkey_hash = crate::cip8::blake2b_224(&node_payment_vk);
    let user_pubkey_hash =
        crate::cip8::blake2b_224(state.ed25519_key.verifying_key().as_bytes());

    // The notes pay for the payout and the transaction fee.
    let outflow = input
        .amount
        .checked_add(FEE_LOVELACE)
        .ok_or("withdrawal amount is too large")?;
    let selected = state
        .store
        .select_notes(
            &input.network,
            &withdraw_policy_id,
            &withdraw_asset_name,
            outflow,
        )
        .map_err(|e| e.to_string())?;
    let total_selected: u64 = selected.iter().map(|s| s.note.amount).sum();
    let note_change = total_selected - outflow;

    // Pick vault UTxOs that hold only lovelace, until they cover the
    // outflow. The rest goes back to the vault.
    let (vault_inputs, vault_total) = {
        let provider_guard = state.provider.read().await;
        let provider = provider_guard
            .as_ref()
            .ok_or("no Cardano provider configured")?;
        let utxos = provider
            .get_address_utxos(&script_addr)
            .await
            .map_err(|e| e.to_string())?;

        let mut inputs = Vec::new();
        let mut total = 0u64;
        for utxo in utxos {
            if total >= outflow {
                break;
            }
            let [amount] = utxo.amount.as_slice() else {
                continue;
            };
            if amount.unit != "lovelace" {
                continue;
            }
            let Ok(lovelace) = amount.quantity.parse::<u64>() else {
                continue;
            };
            inputs.push((utxo.tx_hash, utxo.output_index as u32));
            total += lovelace;
        }

        if total < outflow {
            return Err(format!(
                "vault holds {total} lovelace in plain UTxOs; need {outflow}"
            ));
        }
        (inputs, total)
    };

    let (tx_cbor, tx_hash) = crate::cardano_tx::build_withdraw_tx(
        &crate::cardano_tx::WithdrawTxParams {
            script_inputs: &vault_inputs,
            total_input_lovelace: vault_total,
            destination_address: &input.destination_address,
            withdraw_amount_lovelace: input.amount,
            script_address: &script_addr,
            fee_lovelace: FEE_LOVELACE,
            node_pubkey_hash: &node_pubkey_hash,
            user_pubkey_hash: &user_pubkey_hash,
        },
    )
    .map_err(|e| format!("tx build: {e}"))?;

    // Blind the note change (and save the blinding factors) before the
    // request leaves the wallet.
    let (change_outputs, pending_change) = crate::notes::blind_new_notes(
        &state.store,
        &input.network,
        delegate_pk,
        &[(withdraw_asset, note_change)],
    )?;

    let withdraw_req = mugraph_core::types::WithdrawRequest {
        notes: selected.iter().map(|s| s.note.clone()).collect(),
        change_outputs,
        tx_cbor: hex::encode(&tx_cbor),
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
    let change_notes =
        crate::notes::finish_notes(client, pending_change, &resp.change_notes)
            .await?;
    for note in &change_notes {
        state
            .store
            .finalize_note(&input.network, note, NoteStatus::Available, now)
            .map_err(|e| e.to_string())?;
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
        change_notes: change_notes.len(),
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
    let (new_pk, script_addr, payment_vk_hex) =
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
    if let Some(ref vk_hex) = payment_vk_hex {
        let vk_bytes = hex::decode(vk_hex)
            .map_err(|e| format!("invalid node payment_vk hex: {e}"))?;
        state
            .store
            .set_node_payment_vk(&network, &vk_bytes)
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
    let note = &stored.note;
    let asset = mugraph_core::types::Asset {
        policy_id: note.policy_id,
        asset_name: note.asset_name,
    };

    let clients = state.node_clients.read().await;
    let client = clients.get(&network).ok_or("no node client")?;
    let new_notes = crate::notes::refresh_through_node(
        &state.store,
        &network,
        client,
        std::slice::from_ref(note),
        &[(asset, note.amount)],
    )
    .await
    .map_err(|e| format!("refresh failed — note may be double-spent: {e}"))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    state
        .store
        .update_note_status(&network, &nonce, NoteStatus::Spent)
        .map_err(|e| e.to_string())?;
    for new_note in &new_notes {
        state
            .store
            .finalize_note(&network, new_note, NoteStatus::Available, now)
            .map_err(|e| e.to_string())?;
    }

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
            setup_complete: true,
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

    #[test]
    fn unix_to_iso8601_epoch() {
        assert_eq!(unix_to_iso8601(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn unix_to_iso8601_known_date() {
        // 2026-04-15T12:00:00Z = 1776254400
        assert_eq!(unix_to_iso8601(1_776_254_400), "2026-04-15T12:00:00Z");
    }

    #[test]
    fn unix_to_iso8601_leap_year() {
        // 2024-02-29T00:00:00Z = 1709164800
        assert_eq!(unix_to_iso8601(1_709_164_800), "2024-02-29T00:00:00Z");
    }

    #[test]
    fn parse_asset_defaults() {
        let (pid, aname) = parse_asset(&None, &None);
        assert_eq!(pid, mugraph_core::types::PolicyId::zero());
        assert!(aname.is_empty());
    }

    #[test]
    fn parse_asset_with_values() {
        let pid_hex = hex::encode([0xAA; 28]);
        let (pid, aname) =
            parse_asset(&Some(pid_hex), &Some("USDM".to_string()));
        assert_eq!(pid.0, [0xAA; 28]);
        assert_eq!(aname.as_bytes(), b"USDM");
    }
}
