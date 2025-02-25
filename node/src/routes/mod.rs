use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{Keypair, PublicKey, Request, Response},
};
use rand::thread_rng;

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
        .route("/public_key", get(get_public_key))
        .with_state(Context {
            database: Arc::new(Database::setup("./db")?),
            keypair,
        });

    Ok(router)
}

pub async fn health() -> &'static str {
    "OK"
}

async fn get_public_key(State(Context { keypair, .. }): State<Context>) -> Json<PublicKey> {
    Json(keypair.public_key)
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
        Request::Emit { asset_id, amount } => {
            let mut rng = thread_rng();
            match emit_note(&keypair, asset_id, amount, &mut rng) {
                Ok(note) => Json(Response::Emit(note)),
                Err(e) => Json(Response::Error {
                    reason: e.to_string(),
                }),
            }
        }
    }
}
