use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};

use crate::network::CardanoNetwork;

mod blockfrost;
mod common;
mod maestro;

pub use common::ProtocolParams;

/// Cardano provider abstraction for UTxO queries and transaction submission
#[derive(Debug, Clone)]
pub enum Provider {
    Blockfrost(BlockfrostProvider),
    Maestro(MaestroProvider),
}

/// Blockfrost provider configuration
#[derive(Debug, Clone)]
pub struct BlockfrostProvider {
    pub api_key: String,
    pub base_url: String,
    pub network: String,
    client: reqwest::Client,
}

/// Maestro provider configuration
#[derive(Debug, Clone)]
pub struct MaestroProvider {
    pub api_key: String,
    pub base_url: String,
    pub network: String,
    client: reqwest::Client,
}

/// UTxO information from the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoInfo {
    pub tx_hash: String,
    pub output_index: u16,
    pub address: String,
    pub amount: Vec<AssetAmount>,
    pub datum_hash: Option<String>,
    /// Raw CBOR hex for inline or referenced datum (if available)
    pub datum: Option<String>,
    pub script_ref: Option<String>,
    /// Block height where this UTxO was created (for confirm depth checks)
    pub block_height: Option<u64>,
}

/// Asset amount (ADA or other tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAmount {
    pub unit: String,
    pub quantity: String,
}

/// Transaction submission response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResponse {
    pub tx_hash: String,
}

/// Current blockchain tip information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainTip {
    pub slot: u64,
    pub hash: String,
    pub block_height: u64,
}

/// Chain observation status for a transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TxSettlementState {
    NotFound,
    Confirming,
    Confirmed,
    Invalidated,
}

/// Deterministic tx observation snapshot used by reconciler/lifecycle logic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxChainObservation {
    pub tx_hash: String,
    pub tx_block_height: Option<u64>,
    pub tip_height: u64,
    pub confirmations: u64,
    pub state: TxSettlementState,
}

impl Provider {
    /// Create a new provider based on configuration
    pub fn new(
        provider_type: &str,
        api_key: String,
        network: String,
        custom_url: Option<String>,
    ) -> Result<Self> {
        if api_key.trim().is_empty() {
            return Err(color_eyre::eyre::eyre!(
                "Missing provider API key. Set CARDANO_API_KEY or pass --cardano-api-key"
            ));
        }

        let typed_network = CardanoNetwork::parse(&network).ok();

        match provider_type {
            "blockfrost" => {
                let base_url = custom_url.unwrap_or_else(|| {
                    typed_network
                        .map(|network| {
                            network.blockfrost_base_url().to_string()
                        })
                        .unwrap_or_else(|| {
                            format!(
                                "https://cardano-{}.blockfrost.io/api/v0",
                                network
                            )
                        })
                });

                Ok(Self::Blockfrost(BlockfrostProvider {
                    api_key,
                    base_url,
                    network,
                    client: reqwest::Client::new(),
                }))
            }
            "maestro" => {
                let base_url = custom_url.unwrap_or_else(|| {
                    typed_network
                        .map(|_| "https://api.gomaestro.org/v1".to_string())
                        .unwrap_or_else(|| {
                            "https://api.gomaestro.org/v1".to_string()
                        })
                });

                Ok(Self::Maestro(MaestroProvider {
                    api_key,
                    base_url,
                    network,
                    client: reqwest::Client::new(),
                }))
            }
            _ => Err(color_eyre::eyre::eyre!(
                "Unknown provider type: {}. Use 'blockfrost' or 'maestro'",
                provider_type
            )),
        }
    }

    pub async fn get_utxo(
        &self,
        tx_hash: &str,
        output_index: u16,
    ) -> Result<Option<UtxoInfo>> {
        match self {
            Self::Blockfrost(provider) => {
                provider.get_utxo(tx_hash, output_index).await
            }
            Self::Maestro(provider) => {
                provider.get_utxo(tx_hash, output_index).await
            }
        }
    }

    pub async fn get_address_utxos(
        &self,
        address: &str,
    ) -> Result<Vec<UtxoInfo>> {
        match self {
            Self::Blockfrost(provider) => {
                provider.get_address_utxos(address).await
            }
            Self::Maestro(provider) => {
                provider.get_address_utxos(address).await
            }
        }
    }

    pub async fn submit_tx(&self, tx_cbor: &[u8]) -> Result<SubmitResponse> {
        match self {
            Self::Blockfrost(provider) => provider.submit_tx(tx_cbor).await,
            Self::Maestro(provider) => provider.submit_tx(tx_cbor).await,
        }
    }

    pub async fn get_tip(&self) -> Result<ChainTip> {
        match self {
            Self::Blockfrost(provider) => provider.get_tip().await,
            Self::Maestro(provider) => provider.get_tip().await,
        }
    }

    pub async fn get_protocol_params(&self) -> Result<ProtocolParams> {
        match self {
            Self::Blockfrost(provider) => provider.get_protocol_params().await,
            Self::Maestro(provider) => provider.get_protocol_params().await,
        }
    }

    pub async fn observe_tx_status(
        &self,
        tx_hash: &str,
        finality_target: u64,
        previously_canonical: bool,
    ) -> Result<TxChainObservation> {
        let tip = self.get_tip().await?;
        let tx_block_height = match self {
            Self::Blockfrost(provider) => {
                provider.get_tx_block_height(tx_hash).await?
            }
            Self::Maestro(provider) => {
                provider.get_tx_block_height(tx_hash).await?
            }
        };

        Ok(evaluate_tx_observation(
            tx_hash,
            tx_block_height,
            tip.block_height,
            finality_target,
            previously_canonical,
        ))
    }
}

pub fn evaluate_tx_observation(
    tx_hash: &str,
    tx_block_height: Option<u64>,
    tip_height: u64,
    finality_target: u64,
    previously_canonical: bool,
) -> TxChainObservation {
    match tx_block_height {
        Some(block_height) => {
            let confirmations = if tip_height >= block_height {
                tip_height - block_height + 1
            } else {
                0
            };

            let state = if confirmations >= finality_target {
                TxSettlementState::Confirmed
            } else {
                TxSettlementState::Confirming
            };

            TxChainObservation {
                tx_hash: tx_hash.to_string(),
                tx_block_height: Some(block_height),
                tip_height,
                confirmations,
                state,
            }
        }
        None => {
            let state = if previously_canonical {
                TxSettlementState::Invalidated
            } else {
                TxSettlementState::NotFound
            };

            TxChainObservation {
                tx_hash: tx_hash.to_string(),
                tx_block_height: None,
                tip_height,
                confirmations: 0,
                state,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = Provider::new(
            "blockfrost",
            "test_key".to_string(),
            "preprod".to_string(),
            None,
        );
        assert!(provider.is_ok());
    }

    #[test]
    fn test_maestro_provider_creation() {
        let provider = Provider::new(
            "maestro",
            "test_key".to_string(),
            "preprod".to_string(),
            None,
        );
        assert!(provider.is_ok());
    }

    #[test]
    fn test_invalid_provider() {
        let provider = Provider::new(
            "invalid",
            "test_key".to_string(),
            "preprod".to_string(),
            None,
        );
        assert!(provider.is_err());
    }

    #[test]
    fn test_missing_api_key_is_rejected() {
        let provider = Provider::new(
            "blockfrost",
            "".to_string(),
            "preprod".to_string(),
            None,
        );
        assert!(provider.is_err());
    }

    #[test]
    fn test_with_pagination_builds_query() {
        let url = common::with_pagination(
            "https://api.example/addresses/addr/utxos",
            2,
            100,
        );
        assert_eq!(
            url,
            "https://api.example/addresses/addr/utxos?page=2&count=100"
        );
    }
}
