use std::net::SocketAddr;

use clap::Parser;
use color_eyre::eyre::Result;
use mugraph_core::{error::Error, types::Keypair};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::warn;

#[derive(Debug, Clone, Parser)]
pub struct Config {
    #[clap(long)]
    pub seed: Option<u64>,

    #[clap(short, long, default_value = "0.0.0.0:9999")]
    pub addr: SocketAddr,

    #[clap(short, long)]
    pub public_key: Option<String>,

    #[clap(short, long)]
    pub secret_key: Option<String>,

    #[clap(short, long)]
    pub under_test: Option<bool>,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rng(&self) -> ChaCha20Rng {
        match self.seed {
            Some(s) => ChaCha20Rng::seed_from_u64(s),
            None => ChaCha20Rng::from_entropy(),
        }
    }

    pub fn keypair(&self) -> Result<Keypair, Error> {
        let pair = Keypair::random(&mut self.rng());

        if self.public_key.is_none() || self.secret_key.is_none() {
            warn!("No keypair provided, using a random one.");
        }

        if self.public_key.is_some() && self.secret_key.is_none() {
            Err(Error::InvalidKey {
                reason: "Keypair contains public key but no private key".to_string(),
            })?;
        }

        let secret_key = match self.public_key.as_ref() {
            Some(s) => serde_json::from_str(s)?,
            None => pair.secret_key,
        };

        let public_key = match self.secret_key.as_ref() {
            Some(p) => serde_json::from_str(p)?,
            None => pair.public_key,
        };

        Ok(Keypair {
            secret_key,
            public_key,
        })
    }
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Config {
            seed: None,
            addr: "0.0.0.0:3000".parse().unwrap(),
            public_key: None,
            secret_key: None,
            under_test: None,
        }
    }
}