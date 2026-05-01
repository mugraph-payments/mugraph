use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::state::{Chain, MineMode, SubmitError, UtxoEntry};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub addr: SocketAddr,
    pub mode: MineMode,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:8090".parse().expect("hard-coded socket addr"),
            mode: MineMode::OnSubmit,
        }
    }
}

pub struct Server {
    config: ServerConfig,
    chain: Arc<Mutex<Chain>>,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        let chain = Arc::new(Mutex::new(Chain::new(config.mode)));
        Self { config, chain }
    }

    pub fn chain(&self) -> Arc<Mutex<Chain>> {
        self.chain.clone()
    }

    pub fn router(&self) -> Router {
        Router::new()
            .route("/blocks/latest", get(blocks_latest))
            .route("/addresses/{address}/utxos", get(address_utxos))
            .route("/txs/{tx_hash}", get(tx_info))
            .route("/txs/{tx_hash}/utxos", get(tx_utxos))
            .route("/tx/submit", post(tx_submit))
            .route("/epochs/latest/parameters", get(epoch_params))
            .route("/scripts/datum/{datum_hash}/cbor", get(datum_cbor))
            .route("/admin/faucet", post(admin_faucet))
            .route("/admin/mine", post(admin_mine))
            .route("/admin/auto_mine", post(admin_auto_mine))
            .route("/admin/state", get(admin_state))
            .route("/admin/reset", post(admin_reset))
            .with_state(self.chain.clone())
    }

    pub async fn run(self) -> color_eyre::Result<()> {
        let listener = tokio::net::TcpListener::bind(self.config.addr).await?;
        tracing::info!(
            "mock-chain listening on {} (mode = {:?})",
            self.config.addr,
            self.config.mode
        );
        axum::serve(listener, self.router()).await?;
        Ok(())
    }
}

type SharedChain = Arc<Mutex<Chain>>;

#[derive(Serialize)]
struct BlockfrostBlock {
    slot: u64,
    hash: String,
    height: u64,
}

async fn blocks_latest(State(chain): State<SharedChain>) -> impl IntoResponse {
    let chain = chain.lock().await;
    let tip = chain.tip();
    Json(BlockfrostBlock {
        slot: tip.slot,
        hash: tip.hash.clone(),
        height: tip.height,
    })
}

#[derive(Serialize)]
struct AddressUtxo {
    tx_hash: String,
    output_index: i32,
    amount: Vec<AssetAmount>,
    data_hash: Option<String>,
    inline_datum: Option<String>,
    reference_script_hash: Option<String>,
    block_height: Option<u64>,
}

#[derive(Serialize)]
struct AssetAmount {
    unit: String,
    quantity: String,
}

fn utxo_to_address_utxo(u: &UtxoEntry) -> AddressUtxo {
    AddressUtxo {
        tx_hash: u.tx_hash.clone(),
        output_index: u.output_index as i32,
        amount: vec![AssetAmount {
            unit: "lovelace".to_string(),
            quantity: u.lovelace.to_string(),
        }],
        data_hash: u.datum_hash.clone(),
        inline_datum: u.inline_datum_cbor.clone(),
        reference_script_hash: None,
        block_height: u.block_height,
    }
}

#[derive(Deserialize)]
struct PaginationParams {
    page: Option<usize>,
    count: Option<usize>,
}

async fn address_utxos(
    State(chain): State<SharedChain>,
    Path(address): Path<String>,
    Query(p): Query<PaginationParams>,
) -> Json<Vec<AddressUtxo>> {
    let chain = chain.lock().await;
    let all = chain.utxos_at(&address);

    let page = p.page.unwrap_or(1).max(1);
    let count = p.count.unwrap_or(100).min(1000);
    let start = (page - 1) * count;
    let slice: Vec<AddressUtxo> = all
        .iter()
        .skip(start)
        .take(count)
        .map(utxo_to_address_utxo)
        .collect();
    Json(slice)
}

#[derive(Serialize)]
struct TxInfo {
    block_height: u64,
    hash: String,
}

async fn tx_info(
    State(chain): State<SharedChain>,
    Path(tx_hash): Path<String>,
) -> Result<Json<TxInfo>, StatusCode> {
    let chain = chain.lock().await;
    let record = chain.tx(&tx_hash).ok_or(StatusCode::NOT_FOUND)?;
    let block_height = record.block_height.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(TxInfo {
        block_height,
        hash: tx_hash,
    }))
}

#[derive(Serialize)]
struct TxUtxos {
    hash: String,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
}

#[derive(Serialize)]
struct TxInput {
    tx_hash: String,
    output_index: i32,
}

#[derive(Serialize)]
struct TxOutput {
    address: String,
    amount: Vec<AssetAmount>,
    output_index: i32,
    data_hash: Option<String>,
    inline_datum: Option<String>,
    reference_script_hash: Option<String>,
}

async fn tx_utxos(
    State(chain): State<SharedChain>,
    Path(tx_hash): Path<String>,
) -> Result<Json<TxUtxos>, StatusCode> {
    let chain = chain.lock().await;
    let record = chain.tx(&tx_hash).ok_or(StatusCode::NOT_FOUND)?;
    let inputs = record
        .inputs
        .iter()
        .map(|(h, i)| TxInput {
            tx_hash: h.clone(),
            output_index: *i as i32,
        })
        .collect();
    let outputs = record
        .outputs
        .iter()
        .map(|u| TxOutput {
            address: u.address.clone(),
            amount: vec![AssetAmount {
                unit: "lovelace".to_string(),
                quantity: u.lovelace.to_string(),
            }],
            output_index: u.output_index as i32,
            data_hash: u.datum_hash.clone(),
            inline_datum: u.inline_datum_cbor.clone(),
            reference_script_hash: None,
        })
        .collect();
    Ok(Json(TxUtxos {
        hash: tx_hash,
        inputs,
        outputs,
    }))
}

async fn tx_submit(
    State(chain): State<SharedChain>,
    body: axum::body::Bytes,
) -> Result<Json<String>, (StatusCode, String)> {
    let mut chain = chain.lock().await;
    match chain.submit(&body) {
        Ok(hash) => Ok(Json(hash)),
        Err(e) => match e {
            SubmitError::Decode(_) => {
                Err((StatusCode::BAD_REQUEST, e.to_string()))
            }
            SubmitError::InputMissing { .. } => {
                Err((StatusCode::BAD_REQUEST, e.to_string()))
            }
            SubmitError::DuplicateTx(_) => {
                Err((StatusCode::CONFLICT, e.to_string()))
            }
        },
    }
}

#[derive(Serialize)]
struct EpochParams {
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

async fn epoch_params() -> Json<EpochParams> {
    // Stable Conway-era preprod-ish defaults; the demo's wallet/node only
    // need these for fee math, not for protocol-level validation.
    Json(EpochParams {
        min_fee_a: "44".to_string(),
        min_fee_b: "155381".to_string(),
        max_tx_size: "16384".to_string(),
        max_val_size: "5000".to_string(),
        key_deposit: "2000000".to_string(),
        pool_deposit: "500000000".to_string(),
        price_mem: "0.0577".to_string(),
        price_step: "0.0000721".to_string(),
        max_tx_ex_mem: "14000000".to_string(),
        max_tx_ex_steps: "10000000000".to_string(),
        coins_per_utxo_size: "4310".to_string(),
    })
}

#[derive(Serialize)]
struct DatumCbor {
    cbor: String,
}

async fn datum_cbor(
    State(chain): State<SharedChain>,
    Path(datum_hash): Path<String>,
) -> Result<Json<DatumCbor>, StatusCode> {
    let chain = chain.lock().await;
    match chain.datum(&datum_hash) {
        Some(cbor) => Ok(Json(DatumCbor { cbor: cbor.clone() })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Deserialize)]
struct FaucetReq {
    address: String,
    lovelace: u64,
}

#[derive(Serialize)]
struct FaucetResp {
    tx_hash: String,
    output_index: u16,
}

async fn admin_faucet(
    State(chain): State<SharedChain>,
    Json(req): Json<FaucetReq>,
) -> Json<FaucetResp> {
    let mut chain = chain.lock().await;
    let utxo = chain.faucet(&req.address, req.lovelace);
    Json(FaucetResp {
        tx_hash: utxo.tx_hash,
        output_index: utxo.output_index,
    })
}

#[derive(Deserialize)]
struct MineReq {
    count: u64,
}

#[derive(Serialize)]
struct MineResp {
    minted: u64,
    tip_height: u64,
}

async fn admin_mine(
    State(chain): State<SharedChain>,
    Json(req): Json<MineReq>,
) -> Json<MineResp> {
    let mut chain = chain.lock().await;
    let minted = chain.mine(req.count);
    Json(MineResp {
        minted,
        tip_height: chain.tip().height,
    })
}

#[derive(Deserialize)]
struct AutoMineReq {
    on: bool,
}

async fn admin_auto_mine(
    State(chain): State<SharedChain>,
    Json(req): Json<AutoMineReq>,
) -> StatusCode {
    let mut chain = chain.lock().await;
    chain.set_mode(if req.on {
        MineMode::OnSubmit
    } else {
        MineMode::Manual
    });
    StatusCode::NO_CONTENT
}

#[derive(Serialize)]
struct AdminState {
    tip_height: u64,
    tip_slot: u64,
    tip_hash: String,
    live_utxos: usize,
    pending_txs: usize,
    confirmed_txs: usize,
    mode: String,
}

async fn admin_state(State(chain): State<SharedChain>) -> Json<AdminState> {
    let chain = chain.lock().await;
    let tip = chain.tip();
    Json(AdminState {
        tip_height: tip.height,
        tip_slot: tip.slot,
        tip_hash: tip.hash.clone(),
        live_utxos: chain.live_utxo_count(),
        pending_txs: chain.pending_count(),
        confirmed_txs: chain.tx_count(),
        mode: format!("{:?}", chain.mode()),
    })
}

async fn admin_reset(State(chain): State<SharedChain>) -> StatusCode {
    let mut chain = chain.lock().await;
    chain.reset();
    StatusCode::NO_CONTENT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn faucet_then_query_address_utxos() {
        let server = Server::new(ServerConfig {
            addr: "127.0.0.1:0".parse().unwrap(),
            mode: MineMode::OnSubmit,
        });
        let chain = server.chain();

        {
            let mut c = chain.lock().await;
            c.faucet("addr_test1abc", 5_000_000);
        }

        let chain = chain.lock().await;
        let live = chain.utxos_at("addr_test1abc");
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].lovelace, 5_000_000);
        assert!(live[0].block_height.is_some());
    }
}
