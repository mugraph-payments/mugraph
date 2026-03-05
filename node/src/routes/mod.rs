use std::sync::Arc;

use axum::{
    Json,
    Router,
    extract::State,
    routing::{get, post},
};
use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{Keypair, Request, Response},
};

mod cross_node;
mod deposit;
mod refresh;
mod withdraw;

pub use cross_node::*;
pub use deposit::*;
pub use refresh::*;
pub use withdraw::*;

use crate::{
    cardano::setup_cardano_wallet,
    config::Config,
    database::{CARDANO_WALLET, Database},
    deposit_monitor::{DepositMonitor, DepositMonitorConfig},
    peer_registry::PeerRegistry,
    provider::Provider,
    reconciler::{RetryPolicy, reconciler_loop},
};

#[derive(Clone)]
pub struct Context {
    keypair: Keypair,
    database: Arc<Database>,
    config: Config,
    peer_registry: Option<Arc<PeerRegistry>>,
}

fn default_database_path() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("MUGRAPH_DB_PATH")
        && !path.trim().is_empty() {
            return std::path::PathBuf::from(path);
        }

    if let Ok(home) = std::env::var("HOME") {
        return std::path::PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("mugraph")
            .join("db.redb");
    }

    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("db.redb")
}

pub async fn router(config: Config) -> Result<Router, Error> {
    let database = Arc::new(Database::setup(default_database_path())?);

    // Run database migrations
    database.migrate()?;

    // Validate trusted peer registry when configured and keep it in memory
    let peer_registry = if let Some(path) = config.xnode_peer_registry_file() {
        let registry = PeerRegistry::load(&path)?;
        registry.validate()?;
        tracing::info!(
            peers = registry.peers.len(),
            path = %path,
            "loaded trusted peer registry"
        );
        Some(Arc::new(registry))
    } else {
        None
    };

    if config.dev_mode() {
        tracing::warn!("dev mode enabled — skipping Cardano wallet, deposit monitor, and reconciler");
    } else {
        // Initialize Cardano wallet on startup
        initialize_cardano_wallet(&config, &database).await?;

        // Start deposit monitor background task
        start_deposit_monitor(&config, database.clone()).await?;

        // Start cross-node reconciler worker for retry/recovery convergence
        start_cross_node_reconciler(database.clone()).await?;
    }

    let keypair = config.keypair()?;

    let router = Router::new()
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .route("/health", get(health))
        .route("/rpc", post(rpc))
        .with_state(Context {
            database,
            keypair,
            config,
            peer_registry,
        });

    Ok(router)
}

/// Initialize Cardano wallet on startup
/// Loads existing wallet or creates a new one with compiled validator
async fn initialize_cardano_wallet(config: &Config, database: &Database) -> Result<(), Error> {
    // Check if wallet already exists
    {
        let read_tx = database.read()?;
        let table = read_tx.open_table(CARDANO_WALLET)?;
        if table.get("wallet")?.is_some() {
            tracing::info!("Cardano wallet already initialized");
            return Ok(());
        }
    }

    tracing::info!("Initializing Cardano wallet...");

    // Get network and optional payment key from config
    let network = config.network();
    let payment_sk = config.payment_sk();

    // Create or load wallet
    let wallet = setup_cardano_wallet(&network, payment_sk.as_deref())
        .await
        .map_err(|e| Error::Internal {
            reason: format!("Failed to setup Cardano wallet: {}", e),
        })?;

    // Store wallet in database
    let write_tx = database.write()?;
    {
        let mut table = write_tx.open_table(CARDANO_WALLET)?;
        table.insert("wallet", &wallet)?;
    }
    write_tx.commit()?;

    tracing::info!(
        "Cardano wallet initialized successfully. Script address: {}",
        wallet.script_address
    );

    Ok(())
}

/// Start the deposit monitor background task
async fn start_deposit_monitor(config: &Config, database: Arc<Database>) -> Result<(), Error> {
    // Create provider for the monitor using config
    let provider = Provider::new(
        &config.provider_type(),
        config.provider_api_key(),
        config.network(),
        config.provider_url(),
    )
    .map_err(|e| Error::Internal {
        reason: format!("Failed to create provider for deposit monitor: {}", e),
    })?;

    // Create monitor configuration from config
    let monitor_config = DepositMonitorConfig {
        confirm_depth: config.deposit_confirm_depth(),
        expiration_blocks: config.deposit_expiration_blocks(),
        min_deposit_value: config.min_deposit_value(),
        revalidation_interval: 60, // 1 minute
    };

    // Create and start monitor
    let monitor = DepositMonitor::new(monitor_config, database, provider);

    // Spawn the monitor as a background task
    tokio::spawn(async move {
        monitor.start().await;
    });

    tracing::info!("Deposit monitor started in background");

    Ok(())
}

async fn start_cross_node_reconciler(database: Arc<Database>) -> Result<(), Error> {
    tokio::spawn(async move {
        reconciler_loop(
            database,
            std::time::Duration::from_secs(5),
            RetryPolicy::default(),
        )
        .await;
    });

    tracing::info!("Cross-node reconciler started in background");
    Ok(())
}

pub async fn health() -> &'static str {
    "OK"
}

#[tracing::instrument(skip_all)]
pub async fn rpc(State(ctx): State<Context>, Json(request): Json<Request>) -> Json<Response> {
    match request {
        Request::Refresh(t) => match refresh(&t, ctx.keypair, &ctx.database) {
            Ok(response) => Json(response),
            Err(e) => Json(Response::Error {
                reason: e.to_string(),
            }),
        },
        Request::Info => {
            // Load cardano script address if available
            let script_address = load_cardano_script_address(&ctx.database).ok();
            Json(Response::Info {
                delegate_pk: ctx.keypair.public_key,
                cardano_script_address: script_address,
            })
        }
        Request::Emit {
            policy_id,
            asset_name,
            amount,
        } => {
            if !ctx.config.dev_mode() {
                return Json(Response::Error {
                    reason: "Emit is only available in dev mode".to_string(),
                });
            }
            let mut rng = rand::rng();
            match emit_note(&ctx.keypair, policy_id, asset_name, amount, &mut rng) {
                Ok(note) => Json(Response::Emit(Box::new(note))),
                Err(e) => Json(Response::Error {
                    reason: e.to_string(),
                }),
            }
        }
        Request::Deposit(deposit_request) => {
            match deposit::handle_deposit(&deposit_request, &ctx).await {
                Ok(response) => Json(response),
                Err(e) => Json(Response::Error {
                    reason: e.to_string(),
                }),
            }
        }
        Request::Withdraw(withdraw_request) => {
            match withdraw::handle_withdraw(&withdraw_request, &ctx).await {
                Ok(response) => Json(response),
                Err(e) => Json(Response::Error {
                    reason: e.to_string(),
                }),
            }
        }
        Request::CrossNodeTransferCreate(request) => {
            match cross_node::handle_create(&request, &ctx) {
                Ok(response) => Json(response),
                Err(e) => Json(Response::Error {
                    reason: e.to_string(),
                }),
            }
        }
        Request::CrossNodeTransferNotify(request) => {
            match cross_node::handle_notify(&request, &ctx) {
                Ok(response) => Json(response),
                Err(e) => Json(Response::Error {
                    reason: e.to_string(),
                }),
            }
        }
        Request::CrossNodeTransferStatus(request) => {
            match cross_node::handle_status(&request, &ctx) {
                Ok(response) => Json(response),
                Err(e) => Json(Response::Error {
                    reason: e.to_string(),
                }),
            }
        }
        Request::CrossNodeTransferAck(request) => match cross_node::handle_ack(&request, &ctx) {
            Ok(response) => Json(response),
            Err(e) => Json(Response::Error {
                reason: e.to_string(),
            }),
        },
    }
}

/// Load Cardano script address from database if wallet exists
fn load_cardano_script_address(database: &Database) -> Result<String, Error> {
    use crate::database::CARDANO_WALLET;

    let read_tx = database.read()?;
    let table = read_tx.open_table(CARDANO_WALLET)?;

    match table.get("wallet")? {
        Some(wallet) => Ok(wallet.value().script_address),
        None => Err(Error::Internal {
            reason: "Cardano wallet not initialized".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use axum::{Json, extract::State};
    use ed25519_dalek::{Signer, SigningKey};
    use mugraph_core::types::{
        Request,
        TransferNoticePayload,
        TransferNoticeStage,
        XNodeAuth,
        XNodeEnvelope,
        XNodeMessageType,
    };

    use super::*;

    fn test_config(xnode_peer_registry_file: Option<String>) -> Config {
        Config::Server {
            addr: "127.0.0.1:9999".parse().unwrap(),
            seed: Some(42),
            secret_key: None,
            cardano_network: "preprod".to_string(),
            cardano_provider: "blockfrost".to_string(),
            cardano_api_key: Some("test".to_string()),
            cardano_provider_url: None,
            cardano_payment_sk: None,
            xnode_peer_registry_file,
            xnode_node_id: "node://b".to_string(),
            deposit_confirm_depth: 15,
            deposit_expiration_blocks: 1440,
            min_deposit_value: Some(1_000_000),
            max_tx_size: 16384,
            max_withdrawal_fee: 2_000_000,
            fee_tolerance_pct: 5,
            dev_mode: false,
        }
    }

    fn write_registry(pk: &SigningKey) -> String {
        let path = std::env::temp_dir().join(format!(
            "mugraph-peer-registry-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let json = format!(
            r#"{{"peers":[{{"node_id":"node://a","endpoint":"https://a.example/rpc","auth_alg":"Ed25519","kid":"k1","public_key_hex":"{}","revoked":false}}]}}"#,
            muhex::encode(pk.verifying_key().to_bytes())
        );
        std::fs::write(&path, json).unwrap();
        path.display().to_string()
    }

    fn now_rfc3339_offset(secs: i64) -> String {
        (chrono::Utc::now() + chrono::Duration::seconds(secs)).to_rfc3339()
    }

    fn sign_notice(
        mut env: XNodeEnvelope<TransferNoticePayload>,
        sk: &SigningKey,
    ) -> XNodeEnvelope<TransferNoticePayload> {
        let mut canonical = env.clone();
        canonical.auth.sig.clear();
        let body = serde_json::to_vec(&canonical).unwrap();
        let mut payload = Vec::with_capacity("mugraph_xnode_auth_v1".len() + body.len());
        payload.extend_from_slice(b"mugraph_xnode_auth_v1");
        payload.extend_from_slice(&body);
        env.auth.sig = muhex::encode(sk.sign(&payload).to_bytes());
        env
    }

    fn test_context() -> Context {
        let db_path = std::env::temp_dir().join(format!(
            "mugraph-rpc-test-{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let database = Arc::new(Database::setup(db_path).unwrap());
        database.migrate().unwrap();

        let signer = SigningKey::from_bytes(&[7u8; 32]);
        let registry_path = write_registry(&signer);
        let config = test_config(Some(registry_path));
        let keypair = config.keypair().unwrap();

        Context {
            keypair,
            database,
            config,
            peer_registry: None,
        }
    }

    #[test]
    fn default_database_path_uses_absolute_home_location() {
        let path = super::default_database_path();
        assert!(path.is_absolute() || std::env::var("HOME").is_err());
        assert!(path.to_string_lossy().contains("db"));
    }

    #[tokio::test]
    async fn rpc_dispatches_cross_node_transfer_notify() {
        let ctx = test_context();
        let signer = SigningKey::from_bytes(&[7u8; 32]);
        let notice = XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferNotice,
            message_id: "mid-1".to_string(),
            transfer_id: "tr-1".to_string(),
            idempotency_key: "ik-1".to_string(),
            correlation_id: "corr-1".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: now_rfc3339_offset(0),
            expires_at: Some(now_rfc3339_offset(120)),
            payload: TransferNoticePayload {
                notice_stage: TransferNoticeStage::Confirmed,
                tx_hash: "abcd".to_string(),
                confirmations: Some(6),
            },
            auth: XNodeAuth {
                alg: "Ed25519".to_string(),
                kid: "k1".to_string(),
                sig: String::new(),
            },
        };
        let request = Request::CrossNodeTransferNotify(sign_notice(notice, &signer));

        let Json(response) = rpc(State(ctx), Json(request)).await;
        assert!(matches!(
            response,
            mugraph_core::types::Response::CrossNodeTransferNotify { accepted: true }
        ));
    }

    #[tokio::test]
    async fn rpc_returns_error_for_invalid_cross_node_envelope() {
        let ctx = test_context();
        let request = Request::CrossNodeTransferNotify(XNodeEnvelope {
            m: "rpc".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferNotice,
            message_id: "mid-1".to_string(),
            transfer_id: "tr-1".to_string(),
            idempotency_key: "ik-1".to_string(),
            correlation_id: "corr-1".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: "2026-02-26T18:00:00Z".to_string(),
            expires_at: Some("2026-02-26T18:05:00Z".to_string()),
            payload: TransferNoticePayload {
                notice_stage: TransferNoticeStage::Confirmed,
                tx_hash: "abcd".to_string(),
                confirmations: Some(6),
            },
            auth: XNodeAuth {
                alg: "Ed25519".to_string(),
                kid: "k1".to_string(),
                sig: "sig".to_string(),
            },
        });

        let Json(response) = rpc(State(ctx), Json(request)).await;
        assert!(matches!(
            response,
            mugraph_core::types::Response::Error { .. }
        ));
    }
}
