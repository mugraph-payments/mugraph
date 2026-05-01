pub mod cardano_tx;
pub mod cip8;
pub mod commands;
pub mod node_client;
pub mod provider;
pub mod store;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use commands::AppState;
use mugraph_core::types::Keypair;
use store::Store;
use tokio::sync::RwLock;

fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("MUGRAPH_WALLET_DATA_DIR") {
        return PathBuf::from(dir);
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mugraph-wallet")
}

/// Build an `AppState` rooted at an explicit data directory. Used by the
/// Tauri runtime via `init_app_state` and by integration tests that want
/// isolated, ephemeral wallet instances.
pub fn init_app_state_at(dir: &std::path::Path) -> Arc<AppState> {
    let db_path = dir.join("wallet.redb");
    std::fs::create_dir_all(db_path.parent().unwrap()).ok();
    let store = Store::open(&db_path).expect("failed to open wallet database");

    // Load or generate keys
    let keypair = match store.get_keypair_bytes("secret_key").unwrap() {
        Some(sk_bytes) => {
            let sk = mugraph_core::types::SecretKey::try_from(sk_bytes)
                .expect("invalid secret key");
            Keypair {
                secret_key: sk,
                public_key: sk.public(),
            }
        }
        None => {
            let kp = Keypair::random(&mut rand::rng());
            store
                .set_keypair_bytes("secret_key", kp.secret_key.as_ref())
                .expect("failed to persist keypair");
            kp
        }
    };

    let ed25519_key = match store.get_keypair_bytes("ed25519_sk").unwrap() {
        Some(bytes) => {
            let arr: [u8; 32] =
                bytes.try_into().expect("invalid ed25519 key length");
            ed25519_dalek::SigningKey::from_bytes(&arr)
        }
        None => {
            let mut rng_bytes = [0u8; 32];
            rand::Fill::fill(&mut rng_bytes, &mut rand::rng());
            let sk = ed25519_dalek::SigningKey::from_bytes(&rng_bytes);
            store
                .set_keypair_bytes("ed25519_sk", sk.as_bytes())
                .expect("failed to persist ed25519 key");
            sk
        }
    };

    let (cardano_payment_sk, cardano_payment_vk) =
        match store.get_cardano_keypair_bytes("payment_sk").unwrap() {
            Some(sk_bytes) => {
                let vk_bytes = store
                    .get_cardano_keypair_bytes("payment_vk")
                    .unwrap()
                    .expect("payment_vk missing");
                let mut sk = [0u8; 32];
                let mut vk = [0u8; 32];
                sk.copy_from_slice(&sk_bytes);
                vk.copy_from_slice(&vk_bytes);
                (sk, vk)
            }
            None => {
                let mut sk = [0u8; 32];
                rand::Fill::fill(&mut sk, &mut rand::rng());
                // Derive the Cardano payment verification key from the signing key
                // using the CSL library (Ed25519 public key derivation)
                let csl_priv =
                    whisky_csl::csl::PrivateKey::from_normal_bytes(&sk)
                        .expect("valid private key bytes");
                let csl_pub = csl_priv.to_public();
                let vk: [u8; 32] = csl_pub
                    .as_bytes()
                    .try_into()
                    .expect("CSL public key is 32 bytes");
                store
                    .set_cardano_keypair_bytes("payment_sk", &sk)
                    .expect("failed to persist cardano sk");
                store
                    .set_cardano_keypair_bytes("payment_vk", &vk)
                    .expect("failed to persist cardano vk");
                (sk, vk)
            }
        };

    // Scan for orphaned blinding factors on startup (Phase 4 requirement).
    // These represent in-flight operations that crashed between blinding and
    // storing the final note. Log warnings so the user is aware.
    for network in &["mainnet", "preprod", "preview"] {
        match store.scan_orphaned_blinding_factors(network) {
            Ok(orphans) if !orphans.is_empty() => {
                eprintln!(
                    "WARNING: {} orphaned blinding factor(s) found on {network}. \
                     These may represent lost funds from crashed operations.",
                    orphans.len()
                );
                for orphan in &orphans {
                    eprintln!("  orphan key: {}", orphan.key);
                }
            }
            _ => {}
        }
    }

    // Try to restore the provider from stored credentials
    let provider = match (
        store.get_provider_config("type").unwrap(),
        store.get_provider_config("api_key").unwrap(),
    ) {
        (Some(ptype), Some(api_key)) => {
            let base_override =
                store.get_provider_config("base_url_override").unwrap();
            let network = store
                .get_config("last_network")
                .unwrap()
                .unwrap_or_else(|| "preprod".to_string());
            provider::CardanoProvider::new(
                &ptype,
                &api_key,
                &network,
                base_override.as_deref(),
            )
            .ok()
        }
        _ => None,
    };

    Arc::new(AppState {
        store,
        keypair,
        ed25519_key,
        cardano_payment_sk,
        cardano_payment_vk,
        node_clients: RwLock::new(HashMap::new()),
        provider: RwLock::new(provider),
    })
}

pub fn init_app_state() -> Arc<AppState> {
    init_app_state_at(&data_dir())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = init_app_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::complete_guided_setup,
            commands::get_wallet_state,
            commands::switch_network,
            commands::create_receive_request,
            commands::import_notes,
            commands::deposit,
            commands::withdraw,
            commands::send,
            commands::refresh_notes,
            commands::sync,
            commands::retry_quarantined,
            commands::discard_quarantined,
            commands::list_funding_utxos,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
