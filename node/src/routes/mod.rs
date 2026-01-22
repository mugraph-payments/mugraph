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

mod refresh;

pub use refresh::*;

use crate::database::Database;

#[derive(Clone)]
pub struct Context {
    keypair: Keypair,
    database: Arc<Database>,
}

pub fn router(keypair: Keypair) -> Result<Router, Error> {
    let router = Router::new()
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .route("/health", get(health))
        .route("/rpc", post(rpc))
        .with_state(Context {
            database: Arc::new(Database::setup("./db")?),
            keypair,
        });

    Ok(router)
}

pub async fn health() -> &'static str {
    "OK"
}

#[tracing::instrument(skip_all)]
pub async fn rpc(
    State(Context { keypair, database }): State<Context>,
    Json(request): Json<Request>,
) -> Json<Response> {
    match request {
        Request::Refresh(t) => match refresh(&t, keypair, &database) {
            Ok(response) => Json(response),
            Err(e) => Json(Response::Error {
                reason: e.to_string(),
            }),
        },
        Request::Info => Json(Response::Info(keypair.public_key)),
        Request::Emit {
            policy_id,
            asset_name,
            amount,
        } => {
            let mut rng = rand::rng();
            match emit_note(&keypair, policy_id, asset_name, amount, &mut rng) {
                Ok(note) => Json(Response::Emit(Box::new(note))),
                Err(e) => Json(Response::Error {
                    reason: e.to_string(),
                }),
            }
        }
    }
}
