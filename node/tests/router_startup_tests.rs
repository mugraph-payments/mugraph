use std::{
    future::Future,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use mugraph_node::{config::Config, database::Database, routes::router};
use tempfile::TempDir;

fn env_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

fn test_config(dev_mode: bool, peer_registry_file: Option<String>) -> Config {
    Config::Server {
        addr: "127.0.0.1:9999".parse().unwrap(),
        seed: Some(42),
        secret_key: None,
        cardano_network: "preprod".to_string(),
        cardano_provider: "blockfrost".to_string(),
        cardano_api_key: None,
        cardano_provider_url: None,
        cardano_payment_sk: None,
        xnode_peer_registry_file: peer_registry_file,
        xnode_node_id: "node://local".to_string(),
        deposit_confirm_depth: 15,
        deposit_expiration_blocks: 1440,
        min_deposit_value: Some(1_000_000),
        max_tx_size: 16_384,
        max_withdrawal_fee: 2_000_000,
        fee_tolerance_pct: 5,
        dev_mode,
    }
}

async fn with_db_path<T, Fut>(path: &Path, f: impl FnOnce() -> Fut) -> T
where
    Fut: Future<Output = T>,
{
    let _guard = env_lock().lock().await;
    let previous = std::env::var_os("MUGRAPH_DB_PATH");
    // SAFETY: tests in this file serialize environment mutation through a
    // process-wide mutex and hold it across the async startup call.
    unsafe {
        std::env::set_var("MUGRAPH_DB_PATH", path);
    }
    let result = f().await;
    match previous {
        Some(value) => unsafe { std::env::set_var("MUGRAPH_DB_PATH", value) },
        None => unsafe { std::env::remove_var("MUGRAPH_DB_PATH") },
    }
    result
}

#[tokio::test(flavor = "current_thread")]
async fn router_starts_in_dev_mode_and_creates_migrated_database() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("router-dev-mode.redb");

    let built = with_db_path(&db_path, || async {
        router(test_config(true, None)).await
    })
    .await;

    let _ =
        built.expect("dev-mode router should start without chain dependencies");
    assert!(
        db_path.exists(),
        "router should create the configured database path"
    );

    let reopened = Database::setup(PathBuf::from(&db_path)).unwrap();
    assert_eq!(reopened.schema_version().unwrap(), 3);
}

#[tokio::test(flavor = "current_thread")]
async fn router_rejects_missing_peer_registry_at_startup() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("router-missing-registry.redb");
    let missing_registry = dir.path().join("missing-peers.json");

    let err = with_db_path(&db_path, || async {
        router(test_config(
            true,
            Some(missing_registry.display().to_string()),
        ))
        .await
        .unwrap_err()
    })
    .await;

    assert!(
        err.to_string().contains("failed to read peer registry"),
        "unexpected startup error: {err}"
    );
}
