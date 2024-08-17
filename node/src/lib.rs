use std::{collections::BTreeMap, sync::Arc};

use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use color_eyre::eyre::Result;
use mugraph_core::{
    crypto::{hash_to_curve, sign_blinded, verify},
    types::{
        request::{v0::Request as V0Request, Request},
        response::{v0::Response as V0Response, Response},
        Keypair, Signature, Transaction, MAX_ATOMS,
    },
};
use rand::prelude::*;
use redb::{backends::InMemoryBackend, Builder, Database, TableDefinition};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    let mut outputs = vec![];

    match request {
        Request::V0(V0Request::Transaction(transaction)) => {
            if let Err(err) = validate_balances(&transaction) {
                return Json(err).into_response();
            }

            for i in 0..MAX_ATOMS {
                let is_input = transaction.input_mask.contains(i as u8);

                if is_input {
                    let read = deps.db.begin_read().unwrap();
                    let table = read.open_table(TABLE).unwrap();

                    if transaction.signatures[i] == Signature::zero() {
                        return Json(Error::InvalidSignature).into_response();
                    }

                    if table
                        .get(transaction.signatures[i].0)
                        .is_ok_and(|x| x.is_some())
                    {
                        return Json(Error::AlreadySpent).into_response();
                    }

                    if verify(
                        &deps.keypair.public_key,
                        transaction.commitments[i].as_ref(),
                        transaction.signatures[i],
                    )
                    .is_err()
                    {
                        return Json(Error::InvalidSignature).into_response();
                    }
                } else {
                    let sig = sign_blinded(
                        &deps.keypair.secret_key,
                        &hash_to_curve(transaction.commitments[i].as_ref()),
                    );

                    outputs.push(sig);
                }
            }
        }
    }

    let versioned: Response = V0Response::Transaction { outputs }.into();
    Json(versioned).into_response()
}

fn validate_balances(transaction: &Transaction) -> Result<(), Error> {
    let mut inputs = BTreeMap::new();
    let mut outputs = BTreeMap::new();

    for i in 0..MAX_ATOMS {
        let index = match transaction.asset_id_indexes.get(i) {
            Some(index) => *index as usize,
            None => continue,
        };

        let amount = transaction.amounts[i];
        let is_input = transaction.input_mask.contains(i as u8);

        if is_input {
            inputs
                .entry(index)
                .and_modify(|x| *x += amount as u128)
                .or_insert(amount as u128);
        } else {
            outputs
                .entry(index)
                .and_modify(|x| *x += amount as u128)
                .or_insert(amount as u128);
        }
    }

    if inputs != outputs {
        return Err(Error::InvalidRequest);
    }

    Ok(())
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
