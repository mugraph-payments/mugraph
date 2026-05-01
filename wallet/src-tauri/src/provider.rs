use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoInfo {
    pub tx_hash: String,
    pub output_index: u16,
    pub address: String,
    pub amount: Vec<AssetAmount>,
    pub datum_hash: Option<String>,
    pub datum: Option<String>,
    pub block_height: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAmount {
    pub unit: String,
    pub quantity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainTip {
    pub slot: u64,
    pub hash: String,
    pub block_height: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResponse {
    pub tx_hash: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("unsupported provider type: {0}")]
    Unsupported(String),
}

#[derive(Clone)]
pub struct CardanoProvider {
    provider_type: String,
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl CardanoProvider {
    pub fn new(
        provider_type: &str,
        api_key: &str,
        network: &str,
        base_url_override: Option<&str>,
    ) -> Result<Self, ProviderError> {
        let base_url = match base_url_override {
            Some(url) => url.to_string(),
            None => match provider_type {
                "blockfrost" => match network {
                    "mainnet" => "https://cardano-mainnet.blockfrost.io/api/v0"
                        .to_string(),
                    "preprod" => "https://cardano-preprod.blockfrost.io/api/v0"
                        .to_string(),
                    "preview" => "https://cardano-preview.blockfrost.io/api/v0"
                        .to_string(),
                    _ => format!(
                        "https://cardano-{network}.blockfrost.io/api/v0"
                    ),
                },
                "maestro" => "https://api.gomaestro.org/v1".to_string(),
                other => {
                    return Err(ProviderError::Unsupported(other.to_string()));
                }
            },
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| ProviderError::Provider(e.to_string()))?;

        Ok(Self {
            provider_type: provider_type.to_string(),
            api_key: api_key.to_string(),
            base_url,
            client,
        })
    }

    fn auth_header(&self) -> (&str, &str) {
        match self.provider_type.as_str() {
            "blockfrost" => ("project_id", &self.api_key),
            "maestro" => ("api-key", &self.api_key),
            _ => ("Authorization", &self.api_key),
        }
    }

    /// Query UTxOs at a given address.
    pub async fn get_address_utxos(
        &self,
        address: &str,
    ) -> Result<Vec<UtxoInfo>, ProviderError> {
        match self.provider_type.as_str() {
            "blockfrost" => self.blockfrost_get_address_utxos(address).await,
            "maestro" => self.maestro_get_address_utxos(address).await,
            _ => Err(ProviderError::Unsupported(self.provider_type.clone())),
        }
    }

    /// Submit a signed transaction.
    pub async fn submit_tx(
        &self,
        tx_cbor: &[u8],
    ) -> Result<SubmitResponse, ProviderError> {
        match self.provider_type.as_str() {
            "blockfrost" => self.blockfrost_submit_tx(tx_cbor).await,
            "maestro" => self.maestro_submit_tx(tx_cbor).await,
            _ => Err(ProviderError::Unsupported(self.provider_type.clone())),
        }
    }

    /// Get current chain tip.
    pub async fn get_tip(&self) -> Result<ChainTip, ProviderError> {
        match self.provider_type.as_str() {
            "blockfrost" => self.blockfrost_get_tip().await,
            "maestro" => self.maestro_get_tip().await,
            _ => Err(ProviderError::Unsupported(self.provider_type.clone())),
        }
    }

    /// Get transaction block height (for confirmation checking).
    pub async fn get_tx_block_height(
        &self,
        tx_hash: &str,
    ) -> Result<Option<u64>, ProviderError> {
        match self.provider_type.as_str() {
            "blockfrost" => self.blockfrost_get_tx_block_height(tx_hash).await,
            "maestro" => self.maestro_get_tx_block_height(tx_hash).await,
            _ => Err(ProviderError::Unsupported(self.provider_type.clone())),
        }
    }

    /// Check confirmation depth of a transaction.
    pub async fn check_confirmations(
        &self,
        tx_hash: &str,
    ) -> Result<u64, ProviderError> {
        let tip = self.get_tip().await?;
        let tx_height = self.get_tx_block_height(tx_hash).await?;
        match tx_height {
            Some(h) if tip.block_height >= h => Ok(tip.block_height - h + 1),
            _ => Ok(0),
        }
    }

    // --- Blockfrost implementation ---

    async fn blockfrost_get_address_utxos(
        &self,
        address: &str,
    ) -> Result<Vec<UtxoInfo>, ProviderError> {
        let url = format!("{}/addresses/{}/utxos", self.base_url, address);
        let (header_name, header_value) = self.auth_header();
        let resp = self
            .client
            .get(&url)
            .header(header_name, header_value)
            .send()
            .await?;

        if resp.status().as_u16() == 404 {
            return Ok(vec![]);
        }
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Provider(format!(
                "Blockfrost UTxO query failed: {text}"
            )));
        }

        let items: Vec<BlockfrostUtxo> = resp.json().await?;
        Ok(items
            .into_iter()
            .map(|u| UtxoInfo {
                tx_hash: u.tx_hash,
                output_index: u.output_index as u16,
                address: u.address.unwrap_or_default(),
                amount: u
                    .amount
                    .into_iter()
                    .map(|a| AssetAmount {
                        unit: a.unit,
                        quantity: a.quantity,
                    })
                    .collect(),
                datum_hash: u.data_hash,
                datum: u.inline_datum,
                block_height: u.block_height,
            })
            .collect())
    }

    async fn blockfrost_submit_tx(
        &self,
        tx_cbor: &[u8],
    ) -> Result<SubmitResponse, ProviderError> {
        let url = format!("{}/tx/submit", self.base_url);
        let (header_name, header_value) = self.auth_header();
        let resp = self
            .client
            .post(&url)
            .header(header_name, header_value)
            .header("Content-Type", "application/cbor")
            .body(tx_cbor.to_vec())
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Provider(format!(
                "Blockfrost submit failed: {text}"
            )));
        }

        let tx_hash: String = resp.json().await?;
        Ok(SubmitResponse { tx_hash })
    }

    async fn blockfrost_get_tip(&self) -> Result<ChainTip, ProviderError> {
        let url = format!("{}/blocks/latest", self.base_url);
        let (header_name, header_value) = self.auth_header();
        let resp = self
            .client
            .get(&url)
            .header(header_name, header_value)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Provider(format!(
                "Blockfrost tip failed: {text}"
            )));
        }

        let block: BlockfrostBlock = resp.json().await?;
        Ok(ChainTip {
            slot: block.slot.unwrap_or(0),
            hash: block.hash,
            block_height: block.height.unwrap_or(0),
        })
    }

    async fn blockfrost_get_tx_block_height(
        &self,
        tx_hash: &str,
    ) -> Result<Option<u64>, ProviderError> {
        let url = format!("{}/txs/{}", self.base_url, tx_hash);
        let (header_name, header_value) = self.auth_header();
        let resp = self
            .client
            .get(&url)
            .header(header_name, header_value)
            .send()
            .await?;

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Provider(format!(
                "Blockfrost tx info failed: {text}"
            )));
        }

        let tx_info: BlockfrostTxInfo = resp.json().await?;
        Ok(Some(tx_info.block_height))
    }

    // --- Maestro implementation ---

    async fn maestro_get_address_utxos(
        &self,
        address: &str,
    ) -> Result<Vec<UtxoInfo>, ProviderError> {
        let url = format!("{}/addresses/{}/utxos", self.base_url, address);
        let (header_name, header_value) = self.auth_header();
        let resp = self
            .client
            .get(&url)
            .header(header_name, header_value)
            .send()
            .await?;

        if resp.status().as_u16() == 404 {
            return Ok(vec![]);
        }
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Provider(format!(
                "Maestro UTxO query failed: {text}"
            )));
        }

        let wrapper: MaestroUtxoResponse = resp.json().await?;
        Ok(wrapper
            .data
            .into_iter()
            .map(|u| UtxoInfo {
                tx_hash: u.tx_hash,
                output_index: u.index as u16,
                address: address.to_string(),
                amount: u
                    .assets
                    .into_iter()
                    .map(|a| AssetAmount {
                        unit: a.unit,
                        quantity: a.amount,
                    })
                    .collect(),
                datum_hash: u.datum.as_ref().and_then(|d| d.hash.clone()),
                datum: u.datum.and_then(|d| d.bytes),
                block_height: None,
            })
            .collect())
    }

    async fn maestro_submit_tx(
        &self,
        tx_cbor: &[u8],
    ) -> Result<SubmitResponse, ProviderError> {
        let url = format!("{}/txmanager", self.base_url);
        let (header_name, header_value) = self.auth_header();
        let resp = self
            .client
            .post(&url)
            .header(header_name, header_value)
            .header("Content-Type", "application/cbor")
            .body(tx_cbor.to_vec())
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Provider(format!(
                "Maestro submit failed: {text}"
            )));
        }

        let tx_hash: String = resp.text().await?;
        Ok(SubmitResponse { tx_hash })
    }

    async fn maestro_get_tip(&self) -> Result<ChainTip, ProviderError> {
        let url = format!("{}/chain-tip", self.base_url);
        let (header_name, header_value) = self.auth_header();
        let resp = self
            .client
            .get(&url)
            .header(header_name, header_value)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Provider(format!(
                "Maestro tip failed: {text}"
            )));
        }

        let wrapper: MaestroTipResponse = resp.json().await?;
        Ok(ChainTip {
            slot: wrapper.data.slot,
            hash: wrapper.data.hash,
            block_height: wrapper.data.height,
        })
    }

    async fn maestro_get_tx_block_height(
        &self,
        tx_hash: &str,
    ) -> Result<Option<u64>, ProviderError> {
        let url = format!("{}/transactions/{}", self.base_url, tx_hash);
        let (header_name, header_value) = self.auth_header();
        let resp = self
            .client
            .get(&url)
            .header(header_name, header_value)
            .send()
            .await?;

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ProviderError::Provider(format!(
                "Maestro tx info failed: {text}"
            )));
        }

        let wrapper: MaestroTxResponse = resp.json().await?;
        Ok(Some(wrapper.data.block_height))
    }
}

// --- Blockfrost response types ---

#[derive(Deserialize)]
struct BlockfrostUtxo {
    tx_hash: String,
    output_index: i32,
    address: Option<String>,
    amount: Vec<BlockfrostAmount>,
    data_hash: Option<String>,
    inline_datum: Option<String>,
    #[serde(default)]
    block_height: Option<u64>,
}

#[derive(Deserialize)]
struct BlockfrostAmount {
    unit: String,
    quantity: String,
}

#[derive(Deserialize)]
struct BlockfrostBlock {
    hash: String,
    height: Option<u64>,
    slot: Option<u64>,
}

#[derive(Deserialize)]
struct BlockfrostTxInfo {
    block_height: u64,
}

// --- Maestro response types ---

#[derive(Deserialize)]
struct MaestroUtxoResponse {
    data: Vec<MaestroUtxo>,
}

#[derive(Deserialize)]
struct MaestroUtxo {
    tx_hash: String,
    index: i32,
    assets: Vec<MaestroAsset>,
    datum: Option<MaestroDatum>,
}

#[derive(Deserialize)]
struct MaestroAsset {
    unit: String,
    amount: String,
}

#[derive(Deserialize)]
struct MaestroDatum {
    hash: Option<String>,
    bytes: Option<String>,
}

#[derive(Deserialize)]
struct MaestroTipResponse {
    data: MaestroTip,
}

#[derive(Deserialize)]
struct MaestroTip {
    slot: u64,
    hash: String,
    height: u64,
}

#[derive(Deserialize)]
struct MaestroTxResponse {
    data: MaestroTxInfo,
}

#[derive(Deserialize)]
struct MaestroTxInfo {
    block_height: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_blockfrost_preprod() {
        let p = CardanoProvider::new("blockfrost", "key123", "preprod", None)
            .unwrap();
        assert!(p.base_url.contains("preprod"));
    }

    #[test]
    fn new_blockfrost_mainnet() {
        let p = CardanoProvider::new("blockfrost", "key123", "mainnet", None)
            .unwrap();
        assert!(p.base_url.contains("mainnet"));
    }

    #[test]
    fn new_maestro() {
        let p =
            CardanoProvider::new("maestro", "key123", "preprod", None).unwrap();
        assert!(p.base_url.contains("gomaestro"));
    }

    #[test]
    fn new_custom_url() {
        let p = CardanoProvider::new(
            "blockfrost",
            "key",
            "preprod",
            Some("http://localhost:8080"),
        )
        .unwrap();
        assert_eq!(p.base_url, "http://localhost:8080");
    }

    #[test]
    fn new_unsupported_provider() {
        let err = CardanoProvider::new("unknown", "key", "preprod", None);
        assert!(err.is_err());
    }

    #[test]
    fn auth_header_blockfrost() {
        let p = CardanoProvider::new("blockfrost", "mykey", "preprod", None)
            .unwrap();
        let (name, val) = p.auth_header();
        assert_eq!(name, "project_id");
        assert_eq!(val, "mykey");
    }

    #[test]
    fn auth_header_maestro() {
        let p =
            CardanoProvider::new("maestro", "mykey", "preprod", None).unwrap();
        let (name, val) = p.auth_header();
        assert_eq!(name, "api-key");
        assert_eq!(val, "mykey");
    }
}
