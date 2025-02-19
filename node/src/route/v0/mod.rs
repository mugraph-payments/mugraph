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
    types::{Keypair, Request, Response, V0Request},
};

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
) -> impl IntoResponse {
    match request {
        Request::V0(V0Request::Refresh(t)) => match refresh_v0(&t, keypair, &database) {
            Ok(response) => Json(Response::V0(response)).into_response(),
            Err(e) => Json(json!({ "error": e.to_string() })).into_response(),
        },
    }
}
