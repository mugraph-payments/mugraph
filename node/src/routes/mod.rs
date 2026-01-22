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

mod deposit;
mod refresh;
mod withdraw;

pub use deposit::*;
pub use refresh::*;
pub use withdraw::*;

use crate::database::Database;

#[derive(Clone)]
pub struct Context {
    keypair: Keypair,
    database: Arc<Database>,
}

pub fn router(keypair: Keypair) -> Result<Router, Error> {
    let database = Arc::new(Database::setup("./db")?);

    // Run database migrations
    database.migrate()?;

    let router = Router::new()
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .route("/health", get(health))
        .route("/rpc", post(rpc))
        .with_state(Context { database, keypair });

    Ok(router)
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
