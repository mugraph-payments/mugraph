use color_eyre::eyre::Result;
use mugraph_core::{crypto, error::Error, types::*};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use reqwest::blocking::Client;
use tracing::info;

#[derive(Debug)]
pub struct Delegate {
    pub rng: ChaCha20Rng,
    pub keypair: Keypair,
    client: Client,
    target: String,
}

impl Delegate {
    pub fn new<R: Rng + CryptoRng>(rng: &mut R, keypair: Keypair, target: String) -> Result<Self, Error> {
        let rng = ChaCha20Rng::seed_from_u64(rng.gen());

        info!(public_key = %keypair.public_key, "Starting delegate");

        let client = Client::new();
        Ok(Self { rng, keypair, client, target })
    }

    #[tracing::instrument(skip_all)]
    pub fn emit(&mut self, asset_id: Hash, amount: u64) -> Result<Note, Error> {
        let mut note = Note {
            delegate: self.keypair.public_key,
            asset_id,
            nonce: Hash::random(&mut self.rng),
            amount,
            signature: Signature::default(),
        };

        let blind = crypto::blind_note(&mut self.rng, &note);
        let signed = crypto::sign_blinded(&self.keypair.secret_key, &blind.point);
        note.signature =
            crypto::unblind_signature(&signed, &blind.factor, &self.keypair.public_key)?;

        Ok(note)
    }

    #[inline(always)]
    #[tracing::instrument(skip_all)]
    pub fn recv_transaction_v0(&mut self, tx: &Transaction) -> Result<V0Response, Error> {
      let target_endpoint = format!("{}/v0/rpc", self.target);
      let request = Request::V0(V0Request::Transaction(tx.clone()));

      let response = self
        .client
        .post(&target_endpoint)
        .json(&request)
        .send()
        .map_err(|err| Error::ServerError {
          reason: err.to_string(),
        })?;

      let response_text = response.text().map_err(|err| Error::ServerError {
        reason: err.to_string(),
      })?;
      let v0_response: V0Response = serde_json::from_str(&response_text).map_err(|_| {
        if response_text.contains("Atom has already been spent") {
          return Error::AlreadySpent {
            signature: tx.signatures[0],
          };
        }
        Error::ServerError {
          reason: response_text,
        }
      })?;

      Ok(v0_response)
    }
}
