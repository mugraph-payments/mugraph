use color_eyre::eyre::{Context, Result};
use serde::Deserialize;

use super::{
    AssetAmount,
    BlockfrostProvider,
    ChainTip,
    SubmitResponse,
    UtxoInfo,
    common::{
        ADDRESS_UTXO_PAGE_SIZE,
        ProtocolParams,
        parse_required,
        send_with_retry,
        with_pagination,
    },
};

impl BlockfrostProvider {
    pub(super) async fn get_tx_block_height(&self, tx_hash: &str) -> Result<Option<u64>> {
        let url = format!("{}/txs/{}", self.base_url, tx_hash);
        let resp = send_with_retry(
            || self.client.get(&url).header("project_id", &self.api_key),
            "Failed to fetch transaction info from Blockfrost",
        )
        .await?;

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }

        let tx_info: BlockfrostTxInfo = resp
            .json()
            .await
            .context("Failed to parse Blockfrost transaction response")?;
        Ok(Some(tx_info.block_height))
    }

    pub(super) async fn get_utxo(
        &self,
        tx_hash: &str,
        output_index: u16,
    ) -> Result<Option<UtxoInfo>> {
        let url = format!("{}/txs/{}/utxos", self.base_url, tx_hash);

        let resp = send_with_retry(
            || self.client.get(&url).header("project_id", &self.api_key),
            "Failed to fetch UTxO from Blockfrost",
        )
        .await?;

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }

        let response: BlockfrostTxUtxos = resp
            .json()
            .await
            .context("Failed to parse Blockfrost response")?;

        let tx_block_height = self.get_tx_block_height(tx_hash).await?;
        let maybe_output = response
            .outputs
            .into_iter()
            .find(|output| output.output_index == output_index as i32);

        if let Some(output) = maybe_output {
            let datum = if let Some(ref datum_hash) = output.data_hash {
                self.fetch_datum_cbor(datum_hash).await?
            } else {
                None
            };

            return Ok(Some(UtxoInfo {
                tx_hash: tx_hash.to_string(),
                output_index: output.output_index as u16,
                address: output.address,
                amount: output
                    .amount
                    .into_iter()
                    .map(|amount| AssetAmount {
                        unit: amount.unit,
                        quantity: amount.quantity,
                    })
                    .collect(),
                datum_hash: output.data_hash,
                datum,
                script_ref: output.reference_script_hash,
                block_height: tx_block_height,
            }));
        }

        Ok(None)
    }

    pub(super) async fn get_address_utxos(&self, address: &str) -> Result<Vec<UtxoInfo>> {
        let base_url = format!("{}/addresses/{}/utxos", self.base_url, address);

        let mut all = Vec::new();
        for page in 1.. {
            let url = with_pagination(&base_url, page, ADDRESS_UTXO_PAGE_SIZE);
            let response: Vec<BlockfrostAddressUtxo> = send_with_retry(
                || self.client.get(&url).header("project_id", &self.api_key),
                "Failed to fetch address UTxOs from Blockfrost",
            )
            .await?
            .json()
            .await
            .context("Failed to parse Blockfrost response")?;

            if response.is_empty() {
                break;
            }

            let page_len = response.len();
            all.extend(response);
            if page_len < ADDRESS_UTXO_PAGE_SIZE {
                break;
            }
        }

        let mut results = Vec::with_capacity(all.len());
        for utxo in all {
            let block_height = match utxo.block_height {
                Some(height) => Some(height),
                None => {
                    let tx_url = format!("{}/txs/{}", self.base_url, utxo.tx_hash);
                    let tx_info: BlockfrostTxInfo = send_with_retry(
                        || self.client.get(&tx_url).header("project_id", &self.api_key),
                        "Failed to fetch transaction info from Blockfrost",
                    )
                    .await?
                    .json()
                    .await
                    .context("Failed to parse Blockfrost transaction response")?;
                    Some(tx_info.block_height)
                }
            };

            results.push(UtxoInfo {
                tx_hash: utxo.tx_hash,
                output_index: utxo.output_index as u16,
                address: address.to_string(),
                amount: utxo
                    .amount
                    .into_iter()
                    .map(|amount| AssetAmount {
                        unit: amount.unit,
                        quantity: amount.quantity,
                    })
                    .collect(),
                datum_hash: utxo.data_hash,
                datum: None,
                script_ref: utxo.reference_script_hash,
                block_height,
            });
        }

        Ok(results)
    }

    pub(super) async fn submit_tx(&self, tx_cbor: &[u8]) -> Result<SubmitResponse> {
        let url = format!("{}/tx/submit", self.base_url);
        let response = send_with_retry(
            || {
                self.client
                    .post(&url)
                    .header("project_id", &self.api_key)
                    .header("Content-Type", "application/cbor")
                    .body(tx_cbor.to_vec())
            },
            "Failed to submit transaction to Blockfrost",
        )
        .await?;

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

    pub(super) async fn get_tip(&self) -> Result<ChainTip> {
        let url = format!("{}/blocks/latest", self.base_url);
        let response: BlockfrostBlock = send_with_retry(
            || self.client.get(&url).header("project_id", &self.api_key),
            "Failed to fetch latest block from Blockfrost",
        )
        .await?
        .json()
        .await
        .context("Failed to parse Blockfrost block response")?;

        Ok(ChainTip {
            slot: response.slot,
            hash: response.hash,
            block_height: response.height,
        })
    }

    pub(super) async fn get_protocol_params(&self) -> Result<ProtocolParams> {
        let url = format!("{}/epochs/latest/parameters", self.base_url);
        let response: BlockfrostEpochParams = send_with_retry(
            || self.client.get(&url).header("project_id", &self.api_key),
            "Failed to fetch protocol params from Blockfrost",
        )
        .await?
        .json()
        .await
        .context("Failed to parse protocol params response")?;

        Ok(ProtocolParams {
            min_fee_a: parse_required("min_fee_a", &response.min_fee_a)?,
            min_fee_b: parse_required("min_fee_b", &response.min_fee_b)?,
            max_tx_size: parse_required("max_tx_size", &response.max_tx_size)?,
            max_val_size: parse_required("max_val_size", &response.max_val_size)?,
            key_deposit: parse_required("key_deposit", &response.key_deposit)?,
            pool_deposit: parse_required("pool_deposit", &response.pool_deposit)?,
            price_mem: parse_required("price_mem", &response.price_mem)?,
            price_step: parse_required("price_step", &response.price_step)?,
            max_tx_ex_mem: parse_required("max_tx_ex_mem", &response.max_tx_ex_mem)?,
            max_tx_ex_steps: parse_required("max_tx_ex_steps", &response.max_tx_ex_steps)?,
            coins_per_utxo_byte: parse_required(
                "coins_per_utxo_size",
                &response.coins_per_utxo_size,
            )?,
        })
    }

    pub(super) async fn fetch_datum_cbor(&self, datum_hash: &str) -> Result<Option<String>> {
        let url = format!("{}/scripts/datum/{}/cbor", self.base_url, datum_hash);
        let resp = send_with_retry(
            || self.client.get(&url).header("project_id", &self.api_key),
            "Failed to fetch datum CBOR from Blockfrost",
        )
        .await?;

        let status = resp.status();
        if status.as_u16() == 404 {
            return Ok(None);
        }

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(color_eyre::eyre::eyre!(
                "Failed to fetch datum CBOR (status {}): {}",
                status,
                text
            ));
        }

        #[derive(Deserialize)]
        struct DatumCborResponse {
            cbor: String,
        }

        let body: DatumCborResponse = resp
            .json()
            .await
            .context("Failed to parse datum CBOR response")?;
        Ok(Some(body.cbor))
    }
}

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
    output_index: i32,
    amount: Vec<BlockfrostAssetAmount>,
    data_hash: Option<String>,
    reference_script_hash: Option<String>,
    block_height: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct BlockfrostTxInfo {
    block_height: u64,
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

#[cfg(test)]
mod tests {
    use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};
    use serde_json::json;

    use super::*;

    fn provider(base_url: String) -> BlockfrostProvider {
        BlockfrostProvider {
            api_key: "test-key".to_string(),
            base_url,
            network: "preprod".to_string(),
            client: reqwest::Client::new(),
        }
    }

    async fn spawn_datum_mock(ok_status: StatusCode) -> String {
        async fn datum_ok() -> impl IntoResponse {
            (StatusCode::OK, axum::Json(json!({"cbor": "d8799f581c01ff"})))
        }

        async fn datum_missing() -> impl IntoResponse {
            (StatusCode::NOT_FOUND, "missing")
        }

        async fn datum_boom() -> impl IntoResponse {
            (StatusCode::INTERNAL_SERVER_ERROR, "boom")
        }

        let app = match ok_status {
            StatusCode::OK => Router::new().route("/scripts/datum/{datum_hash}/cbor", get(datum_ok)),
            StatusCode::NOT_FOUND => Router::new().route("/scripts/datum/{datum_hash}/cbor", get(datum_missing)),
            _ => Router::new().route("/scripts/datum/{datum_hash}/cbor", get(datum_boom)),
        };

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        format!("http://{addr}")
    }

    #[tokio::test]
    async fn blockfrost_fetch_datum_cbor_returns_none_on_404_and_error_on_non_404() {
        let ok = provider(spawn_datum_mock(StatusCode::OK).await);
        let missing = provider(spawn_datum_mock(StatusCode::NOT_FOUND).await);
        let failing = provider(spawn_datum_mock(StatusCode::INTERNAL_SERVER_ERROR).await);

        assert_eq!(ok.fetch_datum_cbor("hash").await.unwrap(), Some("d8799f581c01ff".to_string()));
        assert!(missing.fetch_datum_cbor("hash").await.unwrap().is_none());

        let err = failing.fetch_datum_cbor("hash").await.unwrap_err();
        assert!(format!("{err}").contains("status 500"));
        assert!(format!("{err}").contains("boom"));
    }
}
