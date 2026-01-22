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

        /// Cardano network (mainnet, preprod, preview, testnet)
        #[clap(long, env = "CARDANO_NETWORK", default_value = "preprod")]
        cardano_network: String,

        /// Cardano provider (blockfrost, maestro)
        #[clap(long, env = "CARDANO_PROVIDER", default_value = "blockfrost")]
        cardano_provider: String,

        /// Cardano provider API key
        #[clap(long, env = "CARDANO_API_KEY")]
        cardano_api_key: Option<String>,

        /// Cardano provider URL (optional, for custom endpoints)
        #[clap(long, env = "CARDANO_PROVIDER_URL")]
        cardano_provider_url: Option<String>,

        /// Optional Cardano payment signing key to import (hex encoded)
        #[clap(long, env = "CARDANO_PAYMENT_SK")]
        cardano_payment_sk: Option<String>,

        /// Number of blocks for deposit confirmation depth (default: 15)
        #[clap(long, env = "DEPOSIT_CONFIRM_DEPTH", default_value = "15")]
        deposit_confirm_depth: u64,

        /// Number of blocks after which unclaimed deposits expire (default: 1440)
        #[clap(long, env = "DEPOSIT_EXPIRATION_BLOCKS", default_value = "1440")]
        deposit_expiration_blocks: u64,

        /// Minimum deposit value in lovelace
        #[clap(long, env = "MIN_DEPOSIT_VALUE")]
        min_deposit_value: Option<u64>,

        /// Maximum transaction size in bytes for withdrawal CBOR (default: 16384)
        #[clap(long, env = "MAX_TX_SIZE", default_value = "16384")]
        max_tx_size: usize,
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
                let key_bytes = muhex::decode(secret_key).map_err(|e| Error::InvalidKey {
                    reason: e.to_string(),
                })?;

                if key_bytes.len() != 32 {
                    return Err(Error::InvalidKey {
                        reason: "Secret key must be 32 bytes".to_string(),
                    });
                }

                let key: [u8; 32] = key_bytes.try_into().expect("Already validated length");
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
