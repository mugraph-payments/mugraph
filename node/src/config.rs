use std::net::SocketAddr;

use clap::Parser;
use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{Keypair, SecretKey},
};

#[derive(Debug, Clone, Parser)]
pub enum Config {
    #[command(about)]
    Server {
        #[clap(short, long, default_value = "0.0.0.0:9999")]
        addr: SocketAddr,

        #[clap(long)]
        seed: Option<u64>,

        #[clap(short, long)]
        secret_key: String,
    },
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

    pub fn keypair(&self) -> Result<Keypair, Error> {
        match self {
            Self::Server { secret_key, .. } => {
                let key: [u8; 32] = muhex::decode(secret_key)
                    .map_err(|e| Error::InvalidKey {
                        reason: e.to_string(),
                    })?
                    .try_into()
                    .map_err(|_| Error::InvalidKey {
                        reason: "Invalid key size".to_string(),
                    })?;
                let secret_key = SecretKey::from(key);

                Ok(Keypair {
                    public_key: secret_key.public(),
                    secret_key,
                })
            }
        }
    }
}
