use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use color_eyre::eyre::Result;
use mugraph_core::{
    crypto::{
        schnorr::{SchnorrPair, SchnorrSignature},
        traits::Pair,
    },
    error::Error,
    types::{Keypair, PublicKey, Request, Response, Signature, V0Request},
};

mod transaction;

use rand::rngs::StdRng;
use serde_json::json;
pub use transaction::*;

use crate::database::Database;

#[derive(Clone)]
pub struct Context<P: Pair> {
    keypair: P,
    database: Arc<Mutex<Database>>,
}

pub fn router(keypair: impl Pair + 'static) -> Result<Router, Error> {
    let router = Router::new()
        .route("/health", get(health))
        .route("/rpc", post(rpc))
        .with_state(Context {
            database: Arc::new(Mutex::new(Database::setup("./db")?)),
            keypair,
        });

    Ok(router)
}

pub async fn health() -> &'static str {
    "OK"
}

#[tracing::instrument(skip_all)]
pub async fn rpc<P: Pair>(
    State(Context { keypair, database }): State<Context<P>>,
    Json(request): Json<Request>,
) -> impl IntoResponse {
    match request {
        Request::V0(V0Request::Transaction(t)) => {
            let mut db = database.lock().unwrap();

            match transaction_v0(&t, &keypair, &mut db) {
                Ok(response) => Json(Response::V0(response)).into_response(),
                Err(e) => Json(json!({ "error": e.to_string() })).into_response(),
            }
        }
    }
}
