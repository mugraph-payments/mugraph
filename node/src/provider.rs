use std::collections::HashMap;

use color_eyre::eyre::{Context, Result};
use serde::{Deserialize, Serialize};

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
    pub datum: Option<String>,
    pub script_ref: Option<String>,
    /// Block height where this UTxO was created (for confirm depth checks)
    pub block_height: Option<u64>,
}

/// Asset amount (ADA or other tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAmount {
    pub unit: String, // "lovelace" for ADA, or policy_id + asset_name for tokens
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

impl Provider {
    /// Create a new provider based on configuration
    pub fn new(
        provider_type: &str,
        api_key: String,
        network: String,
        custom_url: Option<String>,
    ) -> Result<Self> {
        match provider_type {
            "blockfrost" => {
                let base_url = custom_url.unwrap_or_else(|| match network.as_str() {
                    "mainnet" => "https://cardano-mainnet.blockfrost.io/api/v0".to_string(),
                    "preprod" => "https://cardano-preprod.blockfrost.io/api/v0".to_string(),
                    "preview" => "https://cardano-preview.blockfrost.io/api/v0".to_string(),
                    _ => format!("https://cardano-{}.blockfrost.io/api/v0", network),
                });

                Ok(Provider::Blockfrost(BlockfrostProvider {
                    api_key,
                    base_url,
                    network,
                    client: reqwest::Client::new(),
                }))
            }
            "maestro" => {
                let base_url = custom_url.unwrap_or_else(|| match network.as_str() {
                    "mainnet" => "https://api.gomaestro.org/v1".to_string(),
                    "preprod" => "https://api.gomaestro.org/v1".to_string(),
                    _ => "https://api.gomaestro.org/v1".to_string(),
                });

                Ok(Provider::Maestro(MaestroProvider {
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

    /// Fetch UTxO by reference
    pub async fn get_utxo(&self, tx_hash: &str, output_index: u16) -> Result<Option<UtxoInfo>> {
        match self {
            Provider::Blockfrost(p) => p.get_utxo(tx_hash, output_index).await,
            Provider::Maestro(p) => p.get_utxo(tx_hash, output_index).await,
        }
    }

    /// Get all UTxOs at an address
    pub async fn get_address_utxos(&self, address: &str) -> Result<Vec<UtxoInfo>> {
        match self {
            Provider::Blockfrost(p) => p.get_address_utxos(address).await,
            Provider::Maestro(p) => p.get_address_utxos(address).await,
        }
    }

    /// Submit a transaction
    pub async fn submit_tx(&self, tx_cbor: &[u8]) -> Result<SubmitResponse> {
        match self {
            Provider::Blockfrost(p) => p.submit_tx(tx_cbor).await,
            Provider::Maestro(p) => p.submit_tx(tx_cbor).await,
        }
    }

    /// Get current blockchain tip
    pub async fn get_tip(&self) -> Result<ChainTip> {
        match self {
            Provider::Blockfrost(p) => p.get_tip().await,
            Provider::Maestro(p) => p.get_tip().await,
        }
    }

    /// Get protocol parameters
    pub async fn get_protocol_params(&self) -> Result<ProtocolParams> {
        match self {
            Provider::Blockfrost(p) => p.get_protocol_params().await,
            Provider::Maestro(p) => p.get_protocol_params().await,
        }
    }
}

/// Protocol parameters for fee calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolParams {
    pub min_fee_a: u64,
    pub min_fee_b: u64,
    pub max_tx_size: u64,
    pub max_val_size: u64,
    pub key_deposit: u64,
    pub pool_deposit: u64,
    pub price_mem: f64,
    pub price_step: f64,
    pub max_tx_ex_mem: u64,
    pub max_tx_ex_steps: u64,
    pub coins_per_utxo_byte: u64,
}

impl BlockfrostProvider {
    async fn get_utxo(&self, tx_hash: &str, output_index: u16) -> Result<Option<UtxoInfo>> {
        // Fetch UTxO details
        let url = format!("{}/txs/{}/utxos", self.base_url, tx_hash);

        let response: BlockfrostTxUtxos = self
            .client
            .get(&url)
            .header("project_id", &self.api_key)
            .send()
            .await
            .context("Failed to fetch UTxO from Blockfrost")?
            .json()
            .await
            .context("Failed to parse Blockfrost response")?;

        // Fetch transaction info to get block height
        let tx_url = format!("{}/txs/{}", self.base_url, tx_hash);
        let tx_response: BlockfrostTxInfo = self
            .client
            .get(&tx_url)
            .header("project_id", &self.api_key)
            .send()
            .await
            .context("Failed to fetch transaction info from Blockfrost")?
            .json()
            .await
            .context("Failed to parse Blockfrost transaction response")?;

        Ok(response
            .outputs
            .into_iter()
            .find(|o| o.output_index == output_index as i32)
            .map(|o| UtxoInfo {
                tx_hash: tx_hash.to_string(),
                output_index: o.output_index as u16,
                address: o.address,
                amount: o
                    .amount
                    .into_iter()
                    .map(|a| AssetAmount {
                        unit: a.unit,
                        quantity: a.quantity,
                    })
                    .collect(),
                datum_hash: o.data_hash,
                datum: None, // Would need separate call to get datum
                script_ref: o.reference_script_hash,
                block_height: Some(tx_response.block_height),
            }))
    }

    async fn get_address_utxos(&self, address: &str) -> Result<Vec<UtxoInfo>> {
        let url = format!("{}/addresses/{}/utxos", self.base_url, address);

        let response: Vec<BlockfrostAddressUtxo> = self
            .client
            .get(&url)
            .header("project_id", &self.api_key)
            .send()
            .await
            .context("Failed to fetch address UTxOs from Blockfrost")?
            .json()
            .await
            .context("Failed to parse Blockfrost response")?;

        Ok(response
            .into_iter()
            .map(|u| UtxoInfo {
                tx_hash: u.tx_hash,
                output_index: u.output_index as u16,
                address: address.to_string(),
                amount: u
                    .amount
                    .into_iter()
                    .map(|a| AssetAmount {
                        unit: a.unit,
                        quantity: a.quantity,
                    })
                    .collect(),
                datum_hash: u.data_hash,
                datum: None,
                script_ref: u.reference_script_hash,
                block_height: None, // Would need separate query for each tx
            })
            .collect())
    }

    async fn submit_tx(&self, tx_cbor: &[u8]) -> Result<SubmitResponse> {
        let url = format!("{}/tx/submit", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("project_id", &self.api_key)
            .header("Content-Type", "application/cbor")
            .body(tx_cbor.to_vec())
            .send()
            .await
            .context("Failed to submit transaction to Blockfrost")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(color_eyre::eyre::eyre!(
                "Transaction submission failed: {}",
                error_text
            ));
        }

        let tx_hash: String = response
            .json()
            .await
            .context("Failed to parse submission response")?;

        Ok(SubmitResponse { tx_hash })
    }

    async fn get_tip(&self) -> Result<ChainTip> {
        let url = format!("{}/blocks/latest", self.base_url);

        let response: BlockfrostBlock = self
            .client
            .get(&url)
            .header("project_id", &self.api_key)
            .send()
            .await
            .context("Failed to fetch latest block from Blockfrost")?
            .json()
            .await
            .context("Failed to parse Blockfrost block response")?;

        Ok(ChainTip {
            slot: response.slot,
            hash: response.hash,
            block_height: response.height,
        })
    }

    async fn get_protocol_params(&self) -> Result<ProtocolParams> {
        let url = format!("{}/epochs/latest/parameters", self.base_url);

        let response: BlockfrostEpochParams = self
            .client
            .get(&url)
            .header("project_id", &self.api_key)
            .send()
            .await
            .context("Failed to fetch protocol params from Blockfrost")?
            .json()
            .await
            .context("Failed to parse protocol params response")?;

        Ok(ProtocolParams {
            min_fee_a: response.min_fee_a.parse().unwrap_or(0),
            min_fee_b: response.min_fee_b.parse().unwrap_or(0),
            max_tx_size: response.max_tx_size.parse().unwrap_or(16384),
            max_val_size: response.max_val_size.parse().unwrap_or(5000),
            key_deposit: response.key_deposit.parse().unwrap_or(0),
            pool_deposit: response.pool_deposit.parse().unwrap_or(0),
            price_mem: response.price_mem.parse().unwrap_or(0.0),
            price_step: response.price_step.parse().unwrap_or(0.0),
            max_tx_ex_mem: response.max_tx_ex_mem.parse().unwrap_or(14000000),
            max_tx_ex_steps: response.max_tx_ex_steps.parse().unwrap_or(10000000000),
            coins_per_utxo_byte: response.coins_per_utxo_size.parse().unwrap_or(4310),
        })
    }
}

impl MaestroProvider {
    async fn get_utxo(&self, tx_hash: &str, output_index: u16) -> Result<Option<UtxoInfo>> {
        let url = format!(
            "{}/transactions/{}/outputs/{}?order=desc",
            self.base_url, tx_hash, output_index
        );

        let response: MaestroTxOutput = self
            .client
            .get(&url)
            .header("api-key", &self.api_key)
            .send()
            .await
            .context("Failed to fetch UTxO from Maestro")?
            .json()
            .await
            .context("Failed to parse Maestro response")?;

        Ok(Some(UtxoInfo {
            tx_hash: tx_hash.to_string(),
            output_index,
            address: response.address,
            amount: response
                .assets
                .into_iter()
                .map(|a| AssetAmount {
                    unit: a.unit,
                    quantity: a.quantity,
                })
                .collect(),
            datum_hash: response.datum_hash,
            datum: response.datum,
            script_ref: response.reference_script_hash,
            block_height: response.block_height, // Assuming Maestro provides this
        }))
    }

    async fn get_address_utxos(&self, address: &str) -> Result<Vec<UtxoInfo>> {
        let url = format!("{}/addresses/{}/utxos", self.base_url, address);

        let response: Vec<MaestroAddressUtxo> = self
            .client
            .get(&url)
            .header("api-key", &self.api_key)
            .send()
            .await
            .context("Failed to fetch address UTxOs from Maestro")?
            .json()
            .await
            .context("Failed to parse Maestro response")?;

        Ok(response
            .into_iter()
            .map(|u| UtxoInfo {
                tx_hash: u.tx_hash,
                output_index: u.tx_index as u16,
                address: address.to_string(),
                amount: u
                    .assets
                    .into_iter()
                    .map(|a| AssetAmount {
                        unit: a.unit,
                        quantity: a.quantity,
                    })
                    .collect(),
                datum_hash: u.datum_hash,
                datum: None,
                script_ref: u.reference_script_hash,
                block_height: None, // Would need separate query for each tx
            })
            .collect())
    }

    async fn submit_tx(&self, tx_cbor: &[u8]) -> Result<SubmitResponse> {
        let url = format!("{}/transactions", self.base_url);

        let response: MaestroSubmitResponse = self
            .client
            .post(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/cbor")
            .body(tx_cbor.to_vec())
            .send()
            .await
            .context("Failed to submit transaction to Maestro")?
            .json()
            .await
            .context("Failed to parse Maestro response")?;

        Ok(SubmitResponse {
            tx_hash: response.hash,
        })
    }

    async fn get_tip(&self) -> Result<ChainTip> {
        let url = format!("{}/blocks/latest", self.base_url);

        let response: MaestroBlock = self
            .client
            .get(&url)
            .header("api-key", &self.api_key)
            .send()
            .await
            .context("Failed to fetch tip from Maestro")?
            .json()
            .await
            .context("Failed to parse Maestro response")?;

        Ok(ChainTip {
            slot: response.slot,
            hash: response.hash,
            block_height: response.height,
        })
    }

    async fn get_protocol_params(&self) -> Result<ProtocolParams> {
        let url = format!("{}/protocol-params", self.base_url);

        let response: MaestroProtocolParams = self
            .client
            .get(&url)
            .header("api-key", &self.api_key)
            .send()
            .await
            .context("Failed to fetch protocol params from Maestro")?
            .json()
            .await
            .context("Failed to parse Maestro response")?;

        Ok(ProtocolParams {
            min_fee_a: response.min_fee_a.parse().unwrap_or(44),
            min_fee_b: response.min_fee_b.parse().unwrap_or(155381),
            max_tx_size: response.max_tx_size.parse().unwrap_or(16384),
            max_val_size: response.max_val_size.parse().unwrap_or(5000),
            key_deposit: response.key_deposit.parse().unwrap_or(2000000),
            pool_deposit: response.pool_deposit.parse().unwrap_or(500000000),
            price_mem: response.price_mem.parse().unwrap_or(0.0577),
            price_step: response.price_step.parse().unwrap_or(0.0000721),
            max_tx_ex_mem: response.max_tx_ex_mem.parse().unwrap_or(14000000),
            max_tx_ex_steps: response.max_tx_ex_steps.parse().unwrap_or(10000000000),
            coins_per_utxo_byte: response.coins_per_utxo_byte.parse().unwrap_or(4310),
        })
    }
}

// Blockfrost API response types
#[derive(Debug, Deserialize)]
struct BlockfrostTxUtxos {
    outputs: Vec<BlockfrostUtxoOutput>,
}

#[derive(Debug, Deserialize)]
struct BlockfrostUtxoOutput {
    address: String,
    amount: Vec<BlockfrostAssetAmount>,
    output_index: i32,
    data_hash: Option<String>,
    reference_script_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BlockfrostAssetAmount {
    unit: String,
    quantity: String,
}

#[derive(Debug, Deserialize)]
struct BlockfrostAddressUtxo {
    tx_hash: String,
    tx_index: i32,
    output_index: i32,
    amount: Vec<BlockfrostAssetAmount>,
    data_hash: Option<String>,
    reference_script_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BlockfrostTxInfo {
    hash: String,
    block: String,
    block_height: u64,
    slot: u64,
    index: u32,
}

#[derive(Debug, Deserialize)]
struct BlockfrostBlock {
    slot: u64,
    hash: String,
    height: u64,
}

#[derive(Debug, Deserialize)]
struct BlockfrostEpochParams {
    min_fee_a: String,
    min_fee_b: String,
    max_tx_size: String,
    max_val_size: String,
    key_deposit: String,
    pool_deposit: String,
    price_mem: String,
    price_step: String,
    max_tx_ex_mem: String,
    max_tx_ex_steps: String,
    coins_per_utxo_size: String,
}

// Maestro API response types
#[derive(Debug, Deserialize)]
struct MaestroTxOutput {
    address: String,
    assets: Vec<MaestroAssetAmount>,
    datum_hash: Option<String>,
    datum: Option<String>,
    reference_script_hash: Option<String>,
    block_height: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct MaestroAssetAmount {
    unit: String,
    quantity: String,
}

#[derive(Debug, Deserialize)]
struct MaestroAddressUtxo {
    tx_hash: String,
    tx_index: i32,
    assets: Vec<MaestroAssetAmount>,
    datum_hash: Option<String>,
    reference_script_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MaestroSubmitResponse {
    hash: String,
}

#[derive(Debug, Deserialize)]
struct MaestroBlock {
    hash: String,
    height: u64,
    slot: u64,
}

#[derive(Debug, Deserialize)]
struct MaestroProtocolParams {
    min_fee_a: String,
    min_fee_b: String,
    max_tx_size: String,
    max_val_size: String,
    key_deposit: String,
    pool_deposit: String,
    price_mem: String,
    price_step: String,
    max_tx_ex_mem: String,
    max_tx_ex_steps: String,
    coins_per_utxo_byte: String,
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
}
