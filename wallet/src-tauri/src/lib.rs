pub mod commands;
pub mod node_client;
pub mod store;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use commands::AppState;
use mugraph_core::types::Keypair;
use store::Store;
use tokio::sync::RwLock;

fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mugraph-wallet")
}

fn init_app_state() -> Arc<AppState> {
    let db_path = data_dir().join("wallet.redb");
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
                let vk = sk;
                store
                    .set_cardano_keypair_bytes("payment_sk", &sk)
                    .expect("failed to persist cardano sk");
                store
                    .set_cardano_keypair_bytes("payment_vk", &vk)
                    .expect("failed to persist cardano vk");
                (sk, vk)
            }
        };

    Arc::new(AppState {
        store,
        keypair,
        ed25519_key,
        cardano_payment_sk,
        cardano_payment_vk,
        node_clients: RwLock::new(HashMap::new()),
    })
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
