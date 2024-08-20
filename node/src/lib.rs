use std::net::SocketAddr;

use axum::Router;
use clap::Parser;
use color_eyre::eyre::Result;
use mugraph_core::types::Keypair;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

mod context;
mod route;

#[derive(Debug, Clone, Parser)]
pub struct Config {
    #[clap(short, long)]
    pub seed: Option<u64>,

    #[clap(short, long, default_value = "0.0.0.0:9999")]
    pub addr: SocketAddr,

    #[clap(short, long)]
    pub public_key: String,

    #[clap(short, long)]
    pub secret_key: String,
}

impl Config {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rng(&self) -> ChaCha20Rng {
        match self.seed {
            Some(s) => ChaCha20Rng::seed_from_u64(s),
            None => ChaCha20Rng::from_entropy(),
        }
    }

    pub fn keypair(&self) -> Result<Keypair> {
        let secret_key = serde_json::from_str(&self.secret_key)?;
        let public_key = serde_json::from_str(&self.public_key)?;

        Ok(Keypair {
            secret_key,
            public_key,
        })
    }
}

impl Default for Config {
    #[inline]
    fn default() -> Self {
        Self::parse()
    }
}

pub async fn start(config: &Config) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(config.addr).await?;

    axum::serve(
        listener,
        Router::new().nest("v0", route::v0::router(&mut config.rng())?),
    )
    .await?;

    Ok(())
}
