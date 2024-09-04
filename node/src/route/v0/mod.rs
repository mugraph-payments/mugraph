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

mod transaction;

use serde_json::json;
pub use transaction::*;

use crate::{config::Config, database::Database};

#[derive(Clone)]
pub struct Context {
    keypair: Keypair,
    database: Arc<Database>,
}

pub fn router(config: &Config) -> Result<Router, Error> {
    let router = Router::new()
        .route("/health", get(health))
        .route("/rpc", post(rpc))
        .with_state(Context {
            database: Database::setup("./db")?.into(),
            keypair: config.keypair()?,
        });

    Ok(router)
}

pub async fn health() -> &'static str {
    "OK"
}

#[axum::debug_handler]
pub async fn rpc(
    State(Context { keypair, database }): State<Context>,
    Json(request): Json<Request>,
) -> impl IntoResponse {
    match request {
        Request::V0(V0Request::Transaction(t)) => match transaction_v0(&t, keypair, &database) {
            Ok(response) => Json(Response::V0(response)).into_response(),
            Err(e) => Json(json!({ "error": e.to_string() })).into_response(),
        },
    }
}
