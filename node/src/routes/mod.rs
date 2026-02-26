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
    provider::Provider,
};

#[derive(Clone)]
pub struct Context {
    keypair: Keypair,
    database: Arc<Database>,
    config: Config,
}

pub async fn router(config: Config) -> Result<Router, Error> {
    let database = Arc::new(Database::setup("./db")?);

    // Run database migrations
    database.migrate()?;

    // Initialize Cardano wallet on startup
    initialize_cardano_wallet(&config, &database).await?;

    // Start deposit monitor background task
    start_deposit_monitor(&config, database.clone()).await?;

    let keypair = config.keypair()?;

    let router = Router::new()
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .route("/health", get(health))
        .route("/rpc", post(rpc))
        .with_state(Context {
            database,
            keypair,
            config,
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
        Request::CrossNodeTransferCreate(request) => match cross_node::handle_create(&request) {
            Ok(response) => Json(response),
            Err(e) => Json(Response::Error {
                reason: e.to_string(),
            }),
        },
        Request::CrossNodeTransferNotify(request) => match cross_node::handle_notify(&request) {
            Ok(response) => Json(response),
            Err(e) => Json(Response::Error {
                reason: e.to_string(),
            }),
        },
        Request::CrossNodeTransferStatus(request) => match cross_node::handle_status(&request) {
            Ok(response) => Json(response),
            Err(e) => Json(Response::Error {
                reason: e.to_string(),
            }),
        },
        Request::CrossNodeTransferAck(request) => match cross_node::handle_ack(&request) {
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
    use mugraph_core::types::{
        Request, TransferNoticePayload, TransferNoticeStage, XNodeAuth, XNodeEnvelope,
        XNodeMessageType,
    };

    use super::*;

    fn test_config() -> Config {
        Config::Server {
            addr: "127.0.0.1:9999".parse().unwrap(),
            seed: Some(42),
            secret_key: None,
            cardano_network: "preprod".to_string(),
            cardano_provider: "blockfrost".to_string(),
            cardano_api_key: Some("test".to_string()),
            cardano_provider_url: None,
            cardano_payment_sk: None,
            deposit_confirm_depth: 15,
            deposit_expiration_blocks: 1440,
            min_deposit_value: Some(1_000_000),
            max_tx_size: 16384,
            max_withdrawal_fee: 2_000_000,
            fee_tolerance_pct: 5,
        }
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

        let config = test_config();
        let keypair = config.keypair().unwrap();

        Context {
            keypair,
            database,
            config,
        }
    }

    #[tokio::test]
    async fn rpc_dispatches_cross_node_transfer_notify() {
        let ctx = test_context();
        let request = Request::CrossNodeTransferNotify(XNodeEnvelope {
            m: "xnode".to_string(),
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
        assert!(matches!(response, mugraph_core::types::Response::Error { .. }));
    }
}
