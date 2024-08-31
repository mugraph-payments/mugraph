use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{Request, Response, V0Request, V0Response},
};
use rand::prelude::*;

mod transaction;

pub use transaction::*;

use crate::context::Context;

pub fn router<R: CryptoRng + RngCore>(rng: &mut R) -> Result<Router> {
    Ok(Router::new()
        .route("/health", get(health))
        .route("/rpc", post(rpc))
        .with_state(Context::new(rng)?))
}

pub async fn health() -> &'static str {
    "OK"
}

#[axum::debug_handler]
pub async fn rpc(
    State(mut ctx): State<Context>,
    Json(request): Json<Request>,
) -> impl IntoResponse {
    match request {
        Request::V0(V0Request::Transaction(t)) => {
            match transaction_v0(t, ctx.keypair, &ctx.db().unwrap()) {
                Ok(response) => Json(Response::V0(response)).into_response(),
                Err(Error::Multiple { errors }) => {
                    Json(Response::V0(V0Response::Error { errors })).into_response()
                }
                Err(error) => Json(Response::V0(V0Response::Error {
                    errors: vec![error],
                }))
                .into_response(),
            }
        }
    }
}
