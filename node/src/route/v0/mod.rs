use std::sync::Arc;

use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{Keypair, PublicKey, Request, Response, V0Request, V0Response},
};
use rand::thread_rng;

mod refresh;

pub use refresh::*;
use serde_json::json;

use crate::database::Database;

#[derive(Clone)]
pub struct Context {
    keypair: Keypair,
    database: Arc<Database>,
}

pub fn router(keypair: Keypair) -> Result<Router, Error> {
    let router = Router::new()
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
) -> impl IntoResponse {
    match request {
        Request::V0(V0Request::Refresh(t)) => match refresh_v0(&t, keypair, &database) {
            Ok(response) => Json(Response::V0(response)).into_response(),
            Err(e) => Json(json!({ "error": e.to_string() })).into_response(),
        },
        Request::V0(V0Request::Emit { asset_id, amount }) => {
            let mut rng = thread_rng();
            match emit_note(&keypair, asset_id, amount, &mut rng) {
                Ok(note) => Json(Response::V0(V0Response::Emit(note))).into_response(),
                Err(e) => Json(json!({ "error": e.to_string() })).into_response(),
            }
        }
    }
}
