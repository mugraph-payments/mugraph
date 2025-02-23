use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{Hash, Keypair, Note, Request, Response},
};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use reqwest::blocking::Client;
use tracing::info;

#[derive(Debug)]
pub struct Delegate {
    pub rng: ChaCha20Rng,
    pub client: Client,
    pub node_addr: String,
    pub keypair: Keypair,
}

impl Delegate {
    pub fn new<R: Rng + CryptoRng>(
        rng: &mut R,
        keypair: Keypair,
        node_addr: &str,
    ) -> Result<Self, Error> {
        info!(public_key = %keypair.public_key, "Starting delegate");

        let client = Client::builder().build().map_err(|e| Error::NetworkError {
            reason: e.to_string(),
        })?;

        Ok(Self {
            rng: ChaCha20Rng::seed_from_u64(rng.r#gen()),
            client,
            node_addr: node_addr.to_string(),
            keypair,
        })
    }

    #[tracing::instrument(skip_all)]
    pub fn emit(&mut self, asset_id: Hash, amount: u64) -> Result<Note, Error> {
        let request = Request::Emit { asset_id, amount };

        let response = self
            .client
            .post(format!("{}/rpc", self.node_addr))
            .json(&request)
            .send()
            .map_err(|e| Error::NetworkError {
                reason: e.to_string(),
            })?
            .json()
            .map_err(|e| Error::NetworkError {
                reason: e.to_string(),
            })?;

        match response {
            Response::Emit(note) => Ok(note),
            _ => Err(Error::ServerError {
                reason: "Unexpected response type".into(),
            }),
        }
    }
}
