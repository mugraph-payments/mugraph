use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use color_eyre::eyre::Result;
use mugraph_core::{
    crypto::{hash_to_curve, sign_blinded, verify},
    types::*,
};
use rand::prelude::*;
use redb::{backends::InMemoryBackend, Builder, Database, TableDefinition};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Error {
    AlreadySpent,
    InvalidSignature,
    InvalidRequest,
}

// Maps from Commitment to Signature
const TABLE: TableDefinition<[u8; 32], [u8; 32]> = TableDefinition::new("notes");

pub async fn health() -> &'static str {
    "OK"
}

pub async fn send(
    State(deps): State<Arc<Dependencies>>,
    Json(request): Json<Request>,
) -> axum::response::Response {
    let mut signed_outputs = vec![];
    let mut pre_balances = HashMap::new();
    let mut post_balances = HashMap::new();

    match request {
        Request::Simple { inputs, outputs } => {
            for input in inputs {
                pre_balances
                    .entry(input.asset_id)
                    .and_modify(|x| *x += input.amount as u128)
                    .or_insert(input.amount as u128);
                let read = deps.db.begin_read().unwrap();
                let table = read.open_table(TABLE).unwrap();

                if table.get(input.signature.0).unwrap_or(None).is_some() {
                    return Json(Error::AlreadySpent).into_response();
                }

                if verify(
                    &deps.keypair.public_key,
                    input.nonce.as_ref(),
                    input.signature,
                )
                .is_err()
                {
                    return Json(Error::InvalidSignature).into_response();
                }
            }

            for output in outputs {
                post_balances
                    .entry(output.asset_id)
                    .and_modify(|x| *x += output.amount as u128)
                    .or_insert(output.amount as u128);

                let sig = sign_blinded(
                    &deps.keypair.secret_key,
                    &hash_to_curve(output.commitment.0.as_ref()),
                );

                signed_outputs.push(sig);
            }
        }
    }

    if pre_balances != post_balances {
        return Json(Error::InvalidRequest).into_response();
    }

    Json(Response {
        outputs: signed_outputs,
    })
    .into_response()
}

pub struct Dependencies {
    db: Database,
    keypair: Keypair,
}

pub struct Server {
    router: Router,
}

impl Server {
    pub fn new<R: CryptoRng + RngCore>(rng: &mut R) -> Result<Self> {
        let keypair = Keypair::random(rng);

        let db = Builder::new().create_with_backend(InMemoryBackend::new())?;
        let router = Router::new()
            .route("/health", get(health))
            .route("/v0/send", post(send))
            .with_state(Arc::new(Dependencies { db, keypair }));

        Ok(Self { router })
    }

    pub async fn spawn(self) -> Result<()> {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:9999").await?;

        axum::serve(listener, self.router).await?;

        Ok(())
    }
}
