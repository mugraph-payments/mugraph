use std::net::SocketAddr;

use axum::Router;
use clap::Parser;
use color_eyre::eyre::Result;
use rand::prelude::*;
use rand_chacha::{rand_core::CryptoRngCore, ChaCha20Rng};

mod context;
mod route;

#[derive(Debug, Clone, Parser)]
pub struct Config {
    #[clap(short, long)]
    seed: Option<u64>,

    #[clap(short, long, default_value = "0.0.0.0:9999")]
    addr: SocketAddr,
}

impl Config {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rng(&self) -> Box<dyn CryptoRngCore> {
        match self.seed {
            Some(s) => Box::new(ChaCha20Rng::seed_from_u64(s)),
            None => Box::new(thread_rng()),
        }
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
