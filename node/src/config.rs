use std::net::SocketAddr;

use clap::Parser;
use color_eyre::eyre::Result;
use mugraph_core::{
    error::Error,
    types::{Keypair, SecretKey},
};
use rand::{Rng, SeedableRng, rng};
use rand_chacha::ChaCha20Rng;

#[allow(clippy::large_enum_variant)]
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

        /// Maximum withdrawal fee in lovelace (default: 2_000_000)
        #[clap(long, env = "MAX_WITHDRAWAL_FEE", default_value = "2000000")]
        max_withdrawal_fee: u64,

        /// Fee tolerance percentage (0-100, default: 5%)
        #[clap(long, env = "FEE_TOLERANCE_PCT", default_value = "5")]
        fee_tolerance_pct: u8,
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

    /// Get the Cardano network
    pub fn network(&self) -> String {
        match self {
            Self::Server {
                cardano_network, ..
            } => cardano_network.clone(),
            _ => "preprod".to_string(),
        }
    }

    /// Get the network byte (0 for testnets, 1 for mainnet)
    pub fn network_byte(&self) -> u8 {
        match self.network().as_str() {
            "mainnet" => 1,
            _ => 0,
        }
    }

    /// Get the provider type
    pub fn provider_type(&self) -> String {
        match self {
            Self::Server {
                cardano_provider, ..
            } => cardano_provider.clone(),
            _ => "blockfrost".to_string(),
        }
    }

    /// Get the provider API key
    pub fn provider_api_key(&self) -> String {
        match self {
            Self::Server {
                cardano_api_key, ..
            } => cardano_api_key
                .clone()
                .unwrap_or_else(|| "test_key".to_string()),
            _ => "test_key".to_string(),
        }
    }

    /// Get the provider URL
    pub fn provider_url(&self) -> Option<String> {
        match self {
            Self::Server {
                cardano_provider_url,
                ..
            } => cardano_provider_url.clone(),
            _ => None,
        }
    }

    /// Get the optional payment signing key
    pub fn payment_sk(&self) -> Option<String> {
        match self {
            Self::Server {
                cardano_payment_sk, ..
            } => cardano_payment_sk.clone(),
            _ => None,
        }
    }

    /// Get deposit confirmation depth
    pub fn deposit_confirm_depth(&self) -> u64 {
        match self {
            Self::Server {
                deposit_confirm_depth,
                ..
            } => *deposit_confirm_depth,
            _ => 15,
        }
    }

    /// Get deposit expiration blocks
    pub fn deposit_expiration_blocks(&self) -> u64 {
        match self {
            Self::Server {
                deposit_expiration_blocks,
                ..
            } => *deposit_expiration_blocks,
            _ => 1440,
        }
    }

    /// Get minimum deposit value
    pub fn min_deposit_value(&self) -> u64 {
        match self {
            Self::Server {
                min_deposit_value, ..
            } => min_deposit_value.unwrap_or(1_000_000),
            _ => 1_000_000,
        }
    }

    /// Get maximum withdrawal fee
    pub fn max_withdrawal_fee(&self) -> u64 {
        match self {
            Self::Server {
                max_withdrawal_fee, ..
            } => *max_withdrawal_fee,
            _ => 2_000_000,
        }
    }

    /// Get maximum transaction size
    pub fn max_tx_size(&self) -> usize {
        match self {
            Self::Server { max_tx_size, .. } => *max_tx_size,
            _ => 16384,
        }
    }

    /// Get fee tolerance percentage (0-100)
    pub fn fee_tolerance_pct(&self) -> u8 {
        match self {
            Self::Server {
                fee_tolerance_pct, ..
            } => {
                // Ensure value is between 0 and 100
                (*fee_tolerance_pct).min(100)
            }
            _ => 5,
        }
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
