use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use color_eyre::eyre::Result;
use mugraph_core::{
    crypto,
    error::Error,
    types::{Request, Response, Signature, Transaction, V0Request, V0Response},
};
use rand::prelude::*;

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

#[inline]
pub fn transaction_v0(
    transaction: Transaction,
    ctx: &mut Context,
) -> Result<V0Response, Vec<Error>> {
    let mut outputs = vec![];
    let mut errors = vec![];

    for atom in transaction.atoms.iter() {
        match atom.is_input() {
            true => {
                let signature = match atom.signature {
                    Some(s) if transaction.signatures[s as usize] == Signature::zero() => {
                        errors.push(Error::InvalidSignature {
                            reason: "Signature can not be empty".to_string(),
                            signature: Signature::zero(),
                        });

                        Signature::zero()
                    }
                    Some(s) => transaction.signatures[s as usize],
                    None => {
                        errors.push(Error::InvalidAtom {
                            reason: "Atom {} is an input but it is not signed.".into(),
                        });

                        Signature::zero()
                    }
                };

                let table = ctx.db_read().expect("Failed to read database table");

                match crypto::verify(&ctx.keypair.public_key, atom.nonce.as_ref(), signature) {
                    Ok(_) => {}
                    Err(e) => {
                        errors.push(Error::InvalidSignature {
                            reason: e.to_string(),
                            signature,
                        });
                    }
                }

                match table.get(signature.0) {
                    Ok(Some(_)) => {
                        errors.push(Error::AlreadySpent { signature });
                    }
                    Ok(None) => {}
                    Err(e) => {
                        errors.push(Error::ServerError {
                            reason: e.to_string(),
                        });
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

    if !errors.is_empty() {
        return Err(errors);
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
            Err(errors) => Json(Response::V0(V0Response::Error { errors })).into_response(),
        },
    }
}
