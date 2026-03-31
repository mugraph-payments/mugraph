use color_eyre::eyre::{Context, Result};
use serde::Deserialize;

use super::{
    AssetAmount, ChainTip, MaestroProvider, SubmitResponse, UtxoInfo,
    common::{
        ADDRESS_UTXO_PAGE_SIZE, ProtocolParams, parse_required,
        send_with_retry, with_pagination,
    },
};

impl MaestroProvider {
    pub(super) async fn get_tx_block_height(
        &self,
        tx_hash: &str,
    ) -> Result<Option<u64>> {
        let url = format!("{}/transactions/{}", self.base_url, tx_hash);
        let resp = send_with_retry(
            || self.client.get(&url).header("api-key", &self.api_key),
            "Failed to fetch transaction info from Maestro",
        )
        .await?;

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }

        let tx_info: MaestroTxInfo = resp
            .json()
            .await
            .context("Failed to parse Maestro transaction response")?;

        Ok(tx_info.block_height)
    }

    pub(super) async fn get_utxo(
        &self,
        tx_hash: &str,
        output_index: u16,
    ) -> Result<Option<UtxoInfo>> {
        let url = format!(
            "{}/transactions/{}/outputs/{}?order=desc",
            self.base_url, tx_hash, output_index
        );

        let resp = send_with_retry(
            || self.client.get(&url).header("api-key", &self.api_key),
            "Failed to fetch UTxO from Maestro",
        )
        .await?;

        if resp.status().as_u16() == 404 {
            return Ok(None);
        }

        let response: MaestroTxOutput = resp
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
                .map(|amount| AssetAmount {
                    unit: amount.unit,
                    quantity: amount.quantity,
                })
                .collect(),
            datum_hash: response.datum_hash,
            datum: response.datum,
            script_ref: response.reference_script_hash,
            block_height: response.block_height,
        }))
    }

    pub(super) async fn get_address_utxos(
        &self,
        address: &str,
    ) -> Result<Vec<UtxoInfo>> {
        let base_url = format!("{}/addresses/{}/utxos", self.base_url, address);

        let mut all = Vec::new();
        for page in 1.. {
            let url = with_pagination(&base_url, page, ADDRESS_UTXO_PAGE_SIZE);
            let response: Vec<MaestroAddressUtxo> = send_with_retry(
                || self.client.get(&url).header("api-key", &self.api_key),
                "Failed to fetch address UTxOs from Maestro",
            )
            .await?
            .json()
            .await
            .context("Failed to parse Maestro response")?;

            if response.is_empty() {
                break;
            }

            let page_len = response.len();
            all.extend(response);
            if page_len < ADDRESS_UTXO_PAGE_SIZE {
                break;
            }
        }

        Ok(all
            .into_iter()
            .map(|utxo| UtxoInfo {
                tx_hash: utxo.tx_hash,
                output_index: utxo.tx_index as u16,
                address: address.to_string(),
                amount: utxo
                    .assets
                    .into_iter()
                    .map(|amount| AssetAmount {
                        unit: amount.unit,
                        quantity: amount.quantity,
                    })
                    .collect(),
                datum_hash: utxo.datum_hash,
                datum: None,
                script_ref: utxo.reference_script_hash,
                block_height: None,
            })
            .collect())
    }

    pub(super) async fn submit_tx(
        &self,
        tx_cbor: &[u8],
    ) -> Result<SubmitResponse> {
        let url = format!("{}/transactions", self.base_url);
        let response: MaestroSubmitResponse = send_with_retry(
            || {
                self.client
                    .post(&url)
                    .header("api-key", &self.api_key)
                    .header("Content-Type", "application/cbor")
                    .body(tx_cbor.to_vec())
            },
            "Failed to submit transaction to Maestro",
        )
        .await?
        .json()
        .await
        .context("Failed to parse Maestro response")?;

        Ok(SubmitResponse {
            tx_hash: response.hash,
        })
    }

    pub(super) async fn get_tip(&self) -> Result<ChainTip> {
        let url = format!("{}/blocks/latest", self.base_url);
        let response: MaestroBlock = send_with_retry(
            || self.client.get(&url).header("api-key", &self.api_key),
            "Failed to fetch tip from Maestro",
        )
        .await?
        .json()
        .await
        .context("Failed to parse Maestro response")?;

        Ok(ChainTip {
            slot: response.slot,
            hash: response.hash,
            block_height: response.height,
        })
    }

    pub(super) async fn get_protocol_params(&self) -> Result<ProtocolParams> {
        let url = format!("{}/protocol-params", self.base_url);
        let response: MaestroProtocolParams = send_with_retry(
            || self.client.get(&url).header("api-key", &self.api_key),
            "Failed to fetch protocol params from Maestro",
        )
        .await?
        .json()
        .await
        .context("Failed to parse Maestro response")?;

        Ok(ProtocolParams {
            min_fee_a: parse_required("min_fee_a", &response.min_fee_a)?,
            min_fee_b: parse_required("min_fee_b", &response.min_fee_b)?,
            max_tx_size: parse_required("max_tx_size", &response.max_tx_size)?,
            max_val_size: parse_required(
                "max_val_size",
                &response.max_val_size,
            )?,
            key_deposit: parse_required("key_deposit", &response.key_deposit)?,
            pool_deposit: parse_required(
                "pool_deposit",
                &response.pool_deposit,
            )?,
            price_mem: parse_required("price_mem", &response.price_mem)?,
            price_step: parse_required("price_step", &response.price_step)?,
            max_tx_ex_mem: parse_required(
                "max_tx_ex_mem",
                &response.max_tx_ex_mem,
            )?,
            max_tx_ex_steps: parse_required(
                "max_tx_ex_steps",
                &response.max_tx_ex_steps,
            )?,
            coins_per_utxo_byte: parse_required(
                "coins_per_utxo_byte",
                &response.coins_per_utxo_byte,
            )?,
        })
    }
}

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
struct MaestroTxInfo {
    block_height: Option<u64>,
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
