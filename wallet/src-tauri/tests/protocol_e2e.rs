//! End-to-end run of the protocol through the real components: the mock
//! Cardano chain, the node, and two wallets.
//!
//! Wallet A deposits, sends notes to wallet B, and withdraws. After each
//! step, the notes that the wallets hold must equal the lovelace in the
//! vault, because each note is a claim on the vault.

use std::sync::Arc;

use mugraph_wallet_lib::{
    commands::{
        AppState, DepositInput, SendInput, SetupConfig, WithdrawInput,
        complete_guided_setup_impl, deposit_impl, get_wallet_state_impl,
        import_notes_impl, send_impl, withdraw_impl,
    },
    init_app_state_at,
    store::NoteStatus,
};
use serde_json::json;
use tempfile::TempDir;

const NETWORK: &str = "preprod";

async fn spawn(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

async fn spawn_chain() -> String {
    let server = mock_chain::Server::new(mock_chain::ServerConfig {
        addr: "127.0.0.1:0".parse().unwrap(),
        mode: mock_chain::MineMode::OnSubmit,
    });
    spawn(server.router()).await
}

async fn spawn_node(chain_url: &str, db_dir: &TempDir) -> String {
    // SAFETY: this test binary has one test, so no other thread reads the
    // environment at the same time.
    unsafe {
        std::env::set_var("MUGRAPH_DB_PATH", db_dir.path().join("node.redb"));
    }
    let config = mugraph_node::config::Config::Server {
        addr: "127.0.0.1:0".parse().unwrap(),
        seed: Some(7),
        secret_key: None,
        cardano_network: NETWORK.to_string(),
        cardano_provider: "blockfrost".to_string(),
        cardano_api_key: Some("demo".to_string()),
        cardano_provider_url: Some(chain_url.to_string()),
        cardano_payment_sk: None,
        xnode_peer_registry_file: None,
        xnode_node_id: "node://local".to_string(),
        deposit_confirm_depth: 0,
        deposit_expiration_blocks: 1440,
        min_deposit_value: Some(1_000_000),
        max_tx_size: 16384,
        max_withdrawal_fee: 2_000_000,
        fee_tolerance_pct: 5,
        dev_mode: false,
    };
    let keypair = config.keypair().unwrap();
    let app = mugraph_node::routes::router(config, keypair).await.unwrap();
    spawn(app).await
}

async fn wallet(
    label: &str,
    node_url: &str,
    chain_url: &str,
) -> (Arc<AppState>, TempDir) {
    let dir = TempDir::new().unwrap();
    let state = init_app_state_at(dir.path());
    complete_guided_setup_impl(
        SetupConfig {
            label: label.to_string(),
            mainnet_node_url: node_url.to_string(),
            preprod_node_url: node_url.to_string(),
            preview_node_url: node_url.to_string(),
            provider_type: "blockfrost".to_string(),
            provider_api_key: "demo".to_string(),
            provider_base_url_override: Some(chain_url.to_string()),
        },
        state.clone(),
    )
    .await
    .unwrap();
    (state, dir)
}

async fn available(state: &Arc<AppState>) -> Vec<mugraph_core::types::Note> {
    get_wallet_state_impl(NETWORK.to_string(), state.clone())
        .await
        .unwrap()
        .notes
        .into_iter()
        .filter(|n| n.status == NoteStatus::Available)
        .map(|n| n.note)
        .collect()
}

async fn total(state: &Arc<AppState>) -> u64 {
    available(state).await.iter().map(|n| n.amount).sum()
}

async fn vault_lovelace(chain_url: &str, script_address: &str) -> u64 {
    let utxos: Vec<serde_json::Value> =
        reqwest::get(format!("{chain_url}/addresses/{script_address}/utxos"))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
    utxos
        .iter()
        .flat_map(|u| u["amount"].as_array().unwrap().clone())
        .filter(|a| a["unit"] == "lovelace")
        .map(|a| a["quantity"].as_str().unwrap().parse::<u64>().unwrap())
        .sum()
}

#[tokio::test(flavor = "multi_thread")]
async fn deposit_send_import_and_withdraw_keep_the_vault_backed() {
    let chain_url = spawn_chain().await;
    let node_dir = TempDir::new().unwrap();
    let node_url = spawn_node(&chain_url, &node_dir).await;

    let (a, _a_dir) = wallet("A", &node_url, &chain_url).await;
    let (b, _b_dir) = wallet("B", &node_url, &chain_url).await;
    let script_address = a
        .store
        .get_script_address(NETWORK)
        .unwrap()
        .expect("setup stores the vault address");

    // Fund wallet A on the mock chain.
    let funding = mugraph_wallet_lib::cardano_tx::derive_address(
        &a.cardano_payment_vk,
        NETWORK,
    )
    .unwrap();
    let faucet: serde_json::Value = reqwest::Client::new()
        .post(format!("{chain_url}/admin/faucet"))
        .json(&json!({"address": funding, "lovelace": 200_000_000u64}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // 1. Deposit 100 ADA.
    deposit_impl(
        DepositInput {
            network: NETWORK.to_string(),
            utxo_tx_hash: faucet["tx_hash"].as_str().unwrap().to_string(),
            utxo_index: faucet["output_index"].as_u64().unwrap() as u16,
            output_amounts: vec![100_000_000],
            policy_id: None,
            asset_name: None,
        },
        a.clone(),
    )
    .await
    .expect("deposit");

    assert_eq!(total(&a).await, 100_000_000);
    assert!(
        available(&a)
            .await
            .iter()
            .all(|n| n.amount.is_power_of_two())
    );
    assert_eq!(
        vault_lovelace(&chain_url, &script_address).await,
        100_000_000
    );

    // 2. A sends its two smallest notes to B, and B imports them.
    let mut notes = available(&a).await;
    notes.sort_by_key(|n| n.amount);
    let sent: Vec<_> = notes.iter().take(2).cloned().collect();
    let sent_value: u64 = sent.iter().map(|n| n.amount).sum();
    let envelope = send_impl(
        SendInput {
            network: NETWORK.to_string(),
            note_nonces: sent
                .iter()
                .map(|n| muhex::encode(n.nonce.0))
                .collect(),
        },
        a.clone(),
    )
    .await
    .expect("send")
    .envelope;

    let imported = import_notes_impl(envelope.clone(), b.clone())
        .await
        .expect("import");
    assert_eq!(imported.imported, 2);
    assert_eq!(total(&b).await, sent_value);
    assert_eq!(total(&a).await + total(&b).await, 100_000_000);

    // 3. The same notes can not be imported twice: B swapped them for new
    // notes, so the node knows they are spent.
    let (c, _c_dir) = wallet("C", &node_url, &chain_url).await;
    let again = import_notes_impl(envelope, c.clone())
        .await
        .expect("import");
    assert_eq!(again.imported, 0);
    assert_eq!(again.quarantined, 2);
    assert_eq!(total(&c).await, 0);

    // 4. A withdraws 10 ADA. The notes pay for the payout and the fee.
    let before = total(&a).await;
    withdraw_impl(
        WithdrawInput {
            network: NETWORK.to_string(),
            destination_address: funding.clone(),
            amount: 10_000_000,
            policy_id: None,
            asset_name: None,
        },
        a.clone(),
    )
    .await
    .expect("withdraw");

    assert_eq!(total(&a).await, before - 10_200_000);
    let vault = vault_lovelace(&chain_url, &script_address).await;
    assert_eq!(vault, 100_000_000 - 10_200_000);
    assert_eq!(
        total(&a).await + total(&b).await,
        vault,
        "the notes must equal the value in the vault"
    );
}
