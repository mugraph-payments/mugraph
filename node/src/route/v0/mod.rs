use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use color_eyre::eyre::Result;
use mugraph_core::{
    crypto,
    types::{Request, Response, Signature, Transaction, V0Request, V0Response},
};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::context::Context;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, onlyerror::Error)]
pub enum Error {
    #[error("Input has already been spent")]
    AlreadySpent,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid request")]
    InvalidRequest,
    #[error("Database error: {0}")]
    Database(String),
}

impl From<redb::Error> for Error {
    fn from(e: redb::Error) -> Self {
        Error::Database(e.to_string())
    }
}

impl From<redb::StorageError> for Error {
    fn from(e: redb::StorageError) -> Self {
        Error::Database(e.to_string())
    }
}

pub fn router<R: CryptoRng + RngCore>(rng: &mut R) -> Result<Router> {
    Ok(Router::new()
        .route("/health", get(health))
        .route("/rpc", post(rpc))
        .with_state(Context::new(rng)?))
}

pub async fn health() -> &'static str {
    "OK"
}

pub fn transaction_v0(transaction: Transaction, ctx: &mut Context) -> Result<V0Response, Error> {
    let mut outputs = vec![];

    for atom in transaction.atoms.iter() {
        match atom.is_input() {
            true => {
                let signature = atom
                    .signature
                    .map(|s| transaction.signatures[s as usize])
                    .ok_or(Error::InvalidRequest)?;

                let table = ctx.db_read().expect("Failed to read database table");

                if signature == Signature::zero() {
                    error!("Invalid signature, should not be empty");

                    return Err(Error::InvalidSignature);
                }

                crypto::verify(&ctx.keypair.public_key, atom.nonce.as_ref(), signature)
                    .map_err(|_| Error::InvalidSignature)?;

                if table.get(signature.0)?.is_some() {
                    error!("Note has already been spent");

                    return Err(Error::AlreadySpent);
                }
            }
            false => {
                let sig = crypto::sign_blinded(
                    &ctx.keypair.secret_key,
                    &crypto::hash_to_curve(atom.nonce.as_ref()),
                );

                outputs.push(sig);
            }
        }
    }

    Ok(V0Response::Transaction { outputs })
}

#[axum::debug_handler]
pub async fn rpc(
    State(mut ctx): State<Context>,
    Json(request): Json<Request>,
) -> impl IntoResponse {
    match request {
        Request::V0(V0Request::Transaction(t)) => match transaction_v0(t, &mut ctx) {
            Ok(response) => Json(Response::V0(response)).into_response(),
            Err(e) => Json(e).into_response(),
        },
    }
}
