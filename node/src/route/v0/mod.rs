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
    #[error("Invalid request: {reason}")]
    InvalidRequest { reason: String },
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
    let mut errors = vec![];

    for atom in transaction.atoms.iter() {
        match atom.is_input() {
            true => {
                let signature = match atom.signature {
                    Some(s) if transaction.signatures[s as usize] == Signature::zero() => {
                        errors.push(Error::InvalidSignature);
                        continue;
                    }
                    Some(s) => transaction.signatures[s as usize],
                    None => {
                        errors.push(Error::InvalidRequest {
                            reason: "Atom {} is an input but it is not signed.".into(),
                        });

                        continue;
                    }
                };

                let table = ctx.db_read().expect("Failed to read database table");

                match crypto::verify(&ctx.keypair.public_key, atom.nonce.as_ref(), signature) {
                    Ok(_) => {}
                    Err(_) => {
                        errors.push(Error::InvalidSignature);
                    }
                }

                match table.get(signature.0) {
                    Ok(Some(_)) => {
                        errors.push(Error::AlreadySpent);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        errors.push(Error::Database(e.to_string()));
                    }
                }
            }
            false => {
                let sig = crypto::sign_blinded(
                    &ctx.keypair.secret_key,
                    &crypto::hash_to_curve(atom.commitment(&transaction.asset_ids).as_ref()),
                );

                outputs.push(sig);
            }
        }
    }

    if !errors.is_empty() {}

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
