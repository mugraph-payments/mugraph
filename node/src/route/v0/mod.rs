use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{Keypair, Request, Response, V0Request, V0Response},
};

mod transaction;

use redb::Database;
pub use transaction::*;

use crate::{config::Config, database::DB};

#[derive(Clone)]
pub struct Context {
    keypair: Keypair,
    database: Database,
}

pub fn router(config: &Config) -> Result<Router, Error> {
    let router = Router::new()
        .route("/health", get(health))
        .route("/rpc", post(rpc))
        .with_state(Context {
            database: DB::setup("./db")?,
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
            Err(Error::Multiple { errors }) => {
                Json(Response::V0(V0Response::Error { errors })).into_response()
            }
            Err(error) => Json(Response::V0(V0Response::Error {
                errors: vec![error],
            }))
            .into_response(),
        },
    }
}
