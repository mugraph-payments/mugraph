use std::net::SocketAddr;

use clap::Parser;
use color_eyre::eyre::Result;
use mugraph_core::{
    crypto::{schnorr::SchnorrPair, traits::Pair},
    error::Error,
    types::{Keypair, SecretKey},
};
use rand::thread_rng;
use tracing::warn;

#[derive(Debug, Clone, Parser)]
pub struct Config {
    #[clap(short, long, default_value = "0.0.0.0:9999")]
    pub addr: SocketAddr,

    #[clap(long)]
    pub seed: Option<u64>,

    #[clap(short, long)]
    pub public_key: Option<String>,

    #[clap(short, long)]
    pub secret_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self::parse()
    }

    pub fn keypair(&self) -> Result<impl Pair, Error> {
        match (&self.public_key, &self.secret_key) {
            (None, None) => {
                warn!("No keypair provided, using a random one.");
                Ok(SchnorrPair::random(&mut thread_rng()))
            }
            (None, Some(secret)) => {
                let secret_key: SecretKey = serde_json::from_str(secret)?;
                let schnorr_pair = SchnorrPair::new(secret_key.public(), secret_key);

                Ok(schnorr_pair)
            }
            (Some(_), None) => Err(Error::InvalidKey {
                reason: "Keypair contains public key but no private key".to_string(),
            }),
            (Some(public), Some(secret)) => {
                let public_key = serde_json::from_str(public)?;
                let secret_key = serde_json::from_str(secret)?;
                let schnorr_pair = SchnorrPair::new(public_key, secret_key);

                Ok(schnorr_pair)
            }
        }
    }
}
