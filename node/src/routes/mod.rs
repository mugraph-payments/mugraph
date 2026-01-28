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
use redb::ReadableTable;

mod deposit;
mod refresh;
mod withdraw;

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
