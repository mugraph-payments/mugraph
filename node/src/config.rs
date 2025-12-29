use std::net::SocketAddr;

use clap::Parser;
use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{Keypair, SecretKey},
};
use rand::{Rng, SeedableRng, rng};
use rand_chacha::ChaCha20Rng;

#[derive(Debug, Clone, Parser)]
pub enum Config {
    #[command(about)]
    Server {
        #[clap(short, long, default_value = "0.0.0.0:9999")]
        addr: SocketAddr,

        #[clap(long)]
        seed: Option<u64>,

        #[clap(short, long)]
        secret_key: Option<String>,
    },
    #[command(about)]
    GenerateKey,
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
            Self::GenerateKey => {
                let mut rng = ChaCha20Rng::seed_from_u64(rng().random());
                Ok(Keypair::random(&mut rng))
            }
            Self::Server {
                secret_key: Some(secret_key),
                ..
            } => {
                let key_bytes = muhex::decode(secret_key).map_err(|e| {
                    Error::InvalidKey {
                        reason: e.to_string(),
                    }
                })?;

                if key_bytes.len() != 32 {
                    return Err(Error::InvalidKey {
                        reason: "Secret key must be 32 bytes".to_string(),
                    });
                }

                let key: [u8; 32] =
                    key_bytes.try_into().expect("Already validated length");
                let secret_key = SecretKey::from(key);

                Ok(Keypair {
                    public_key: secret_key.public(),
                    secret_key,
                })
            }
            Self::Server { seed, .. } => {
                let mut rng = match seed {
                    Some(seed) => ChaCha20Rng::seed_from_u64(*seed),
                    None => ChaCha20Rng::from_rng(&mut rand::rng()),
                };

                Ok(Keypair::random(&mut rng))
            }
        }
    }
}
