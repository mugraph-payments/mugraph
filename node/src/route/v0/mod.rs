use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use color_eyre::eyre::Result;
use mugraph_core::{
    crypto,
    types::{
        request::{Request, V0Request},
        response::{Response, V0Response},
        Signature,
    },
};
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::context::Context;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Error {
    AlreadySpent,
    InvalidSignature,
    InvalidRequest,
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

pub async fn rpc(
    State(ctx): State<Context>,
    Json(request): Json<Request>,
) -> axum::response::Response {
    let mut outputs = vec![];

    match request {
        Request::V0(V0Request::Transaction(transaction)) => {
            if !transaction.is_balanced() {
                return Json(Error::InvalidRequest).into_response();
            }

            for atom in transaction.atoms.iter() {
                match atom.is_input() {
                    true => {
                        let signature = match atom
                            .signature
                            .and_then(|s| transaction.signatures.get(s as usize).copied())
                        {
                            Some(s) => s,
                            None => return Json(Error::InvalidRequest).into_response(),
                        };

                        let table = ctx.db_read().unwrap();

                        if signature == Signature::zero() {
                            return Json(Error::InvalidSignature).into_response();
                        }

                        if table.get(signature.0).is_ok_and(|x| x.is_some()) {
                            return Json(Error::AlreadySpent).into_response();
                        }

                        if crypto::verify(&ctx.keypair.public_key, atom.nonce.as_ref(), signature)
                            .is_err()
                        {
                            return Json(Error::InvalidSignature).into_response();
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
        }
    }

    let versioned: Response = V0Response::Transaction { outputs }.into();
    Json(versioned).into_response()
}
