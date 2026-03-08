use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use axum::{
    Router,
    body::Bytes,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use mugraph_node::provider::Provider;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppState {
    statuses: Arc<Mutex<Vec<StatusCode>>>,
    hits: Arc<AtomicUsize>,
}

async fn latest_block(State(state): State<AppState>) -> impl IntoResponse {
    state.hits.fetch_add(1, Ordering::SeqCst);

    let status = {
        let mut lock = state.statuses.lock().await;
        if lock.is_empty() {
            StatusCode::OK
        } else {
            lock.remove(0)
        }
    };

    match status {
        StatusCode::OK => (
            StatusCode::OK,
            axum::Json(json!({"slot": 42, "hash": "abc", "height": 10})),
        )
            .into_response(),
        _ => (status, "mock error").into_response(),
    }
}

async fn spawn_mock(statuses: Vec<StatusCode>) -> (String, Arc<AtomicUsize>) {
    let state = AppState {
        statuses: Arc::new(Mutex::new(statuses)),
        hits: Arc::new(AtomicUsize::new(0)),
    };

    let app = Router::new()
        .route("/blocks/latest", get(latest_block))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test server");
    let addr = listener.local_addr().expect("local addr");

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve mock api");
    });

    (format!("http://{}", addr), state.hits)
}

#[derive(Clone)]
struct AddressUtxoState {
    tx_info_hits: Arc<AtomicUsize>,
}

#[derive(Deserialize)]
struct PaginationQuery {
    page: Option<usize>,
}

async fn address_utxos(
    Path(_address): Path<String>,
    Query(query): Query<PaginationQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    match page {
        1 => {
            let mut entries = Vec::new();
            entries.push(json!({
                "tx_hash": "22".repeat(32),
                "output_index": 1,
                "amount": [{"unit": "lovelace", "quantity": "2000000"}],
                "data_hash": null,
                "reference_script_hash": null,
                "block_height": null
            }));
            for idx in 1..100u16 {
                entries.push(json!({
                    "tx_hash": format!("{:064x}", idx),
                    "output_index": idx,
                    "amount": [{"unit": "lovelace", "quantity": "1000000"}],
                    "data_hash": null,
                    "reference_script_hash": null,
                    "block_height": 77
                }));
            }
            axum::Json(serde_json::Value::Array(entries)).into_response()
        }
        2 => axum::Json(json!([
            {
                "tx_hash": "33".repeat(32),
                "output_index": 2,
                "amount": [{"unit": "lovelace", "quantity": "3000000"}],
                "data_hash": null,
                "reference_script_hash": null,
                "block_height": 99
            }
        ]))
        .into_response(),
        _ => axum::Json(json!([])).into_response(),
    }
}

async fn tx_info_for_address_utxos(
    Path(tx_hash): Path<String>,
    State(state): State<AddressUtxoState>,
) -> impl IntoResponse {
    state.tx_info_hits.fetch_add(1, Ordering::SeqCst);
    let block_height = if tx_hash.starts_with("22") { 88 } else { 55 };
    (StatusCode::OK, axum::Json(json!({"block_height": block_height}))).into_response()
}

async fn spawn_address_utxos_mock() -> (String, Arc<AtomicUsize>) {
    let state = AddressUtxoState {
        tx_info_hits: Arc::new(AtomicUsize::new(0)),
    };

    let app = Router::new()
        .route("/addresses/{address}/utxos", get(address_utxos))
        .route("/txs/{tx_hash}", get(tx_info_for_address_utxos))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{}", addr), state.tx_info_hits)
}

async fn submit_ok(body: Bytes) -> impl IntoResponse {
    assert!(!body.is_empty(), "submitted tx body should not be empty");
    (StatusCode::OK, axum::Json(json!("ab".repeat(32))))
}

async fn submit_bad_request() -> impl IntoResponse {
    (StatusCode::BAD_REQUEST, "cbor decode failed")
}

async fn protocol_params_ok() -> impl IntoResponse {
    axum::Json(json!({
        "min_fee_a": "44",
        "min_fee_b": "155381",
        "max_tx_size": "16384",
        "max_val_size": "5000",
        "key_deposit": "2000000",
        "pool_deposit": "500000000",
        "price_mem": "0.0577",
        "price_step": "0.0000721",
        "max_tx_ex_mem": "14000000",
        "max_tx_ex_steps": "10000000000",
        "coins_per_utxo_size": "4310"
    }))
}

async fn protocol_params_bad() -> impl IntoResponse {
    axum::Json(json!({
        "min_fee_a": "oops",
        "min_fee_b": "155381",
        "max_tx_size": "16384",
        "max_val_size": "5000",
        "key_deposit": "2000000",
        "pool_deposit": "500000000",
        "price_mem": "0.0577",
        "price_step": "0.0000721",
        "max_tx_ex_mem": "14000000",
        "max_tx_ex_steps": "10000000000",
        "coins_per_utxo_size": "4310"
    }))
}

async fn spawn_submit_mock(ok: bool) -> String {
    let app = if ok {
        Router::new().route("/tx/submit", post(submit_ok))
    } else {
        Router::new().route("/tx/submit", post(submit_bad_request))
    };

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://{}", addr)
}

async fn spawn_protocol_params_mock(valid: bool) -> String {
    let app = if valid {
        Router::new().route("/epochs/latest/parameters", get(protocol_params_ok))
    } else {
        Router::new().route("/epochs/latest/parameters", get(protocol_params_bad))
    };

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://{}", addr)
}

#[tokio::test]
async fn get_tip_retries_transient_errors_and_recovers() {
    let (url, hits) = spawn_mock(vec![StatusCode::TOO_MANY_REQUESTS, StatusCode::INTERNAL_SERVER_ERROR, StatusCode::OK]).await;

    let provider = Provider::new(
        "blockfrost",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(url),
    )
    .expect("provider");

    let tip = provider.get_tip().await.expect("tip after retries");
    assert_eq!(tip.block_height, 10);
    assert_eq!(hits.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn get_tip_does_not_retry_non_retriable_4xx() {
    let (url, hits) = spawn_mock(vec![StatusCode::BAD_REQUEST, StatusCode::OK]).await;

    let provider = Provider::new(
        "blockfrost",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(url),
    )
    .expect("provider");

    let err = provider.get_tip().await.expect_err("400 must fail fast");
    let msg = format!("{err}");
    assert!(msg.contains("Failed to fetch latest block from Blockfrost"));
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn get_tip_stops_after_max_retries() {
    let (url, hits) = spawn_mock(vec![
        StatusCode::INTERNAL_SERVER_ERROR,
        StatusCode::INTERNAL_SERVER_ERROR,
        StatusCode::INTERNAL_SERVER_ERROR,
        StatusCode::OK,
    ])
    .await;

    let provider = Provider::new(
        "blockfrost",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(url),
    )
    .expect("provider");

    let err = provider
        .get_tip()
        .await
        .expect_err("must stop after configured retries");
    let msg = format!("{err}");
    assert!(msg.contains("status 500"));
    assert_eq!(hits.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn get_address_utxos_paginates_and_backfills_missing_block_heights() {
    let (url, tx_info_hits) = spawn_address_utxos_mock().await;
    let provider = Provider::new(
        "blockfrost",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(url),
    )
    .expect("provider");

    let utxos = provider
        .get_address_utxos("addr_test1qpagination")
        .await
        .expect("address utxos");

    assert_eq!(utxos.len(), 101);
    assert_eq!(utxos[0].tx_hash, "22".repeat(32));
    assert_eq!(utxos[0].block_height, Some(88));
    assert_eq!(utxos[1].block_height, Some(77));
    assert_eq!(utxos[100].tx_hash, "33".repeat(32));
    assert_eq!(utxos[100].block_height, Some(99));
    assert_eq!(tx_info_hits.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn submit_tx_parses_successful_blockfrost_response() {
    let url = spawn_submit_mock(true).await;
    let provider = Provider::new(
        "blockfrost",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(url),
    )
    .expect("provider");

    let response = provider.submit_tx(&[0xde, 0xad, 0xbe, 0xef]).await.expect("submit ok");
    assert_eq!(response.tx_hash, "ab".repeat(32));
}

#[tokio::test]
async fn submit_tx_surfaces_non_success_response_body() {
    let url = spawn_submit_mock(false).await;
    let provider = Provider::new(
        "blockfrost",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(url),
    )
    .expect("provider");

    let err = provider
        .submit_tx(&[0xca, 0xfe])
        .await
        .expect_err("submit should fail");
    let msg = format!("{err}");
    assert!(msg.contains("status 400"));
    assert!(msg.contains("cbor decode failed"));
}

#[tokio::test]
async fn get_protocol_params_parses_numeric_fields() {
    let url = spawn_protocol_params_mock(true).await;
    let provider = Provider::new(
        "blockfrost",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(url),
    )
    .expect("provider");

    let params = provider.get_protocol_params().await.expect("protocol params");
    assert_eq!(params.min_fee_a, 44);
    assert_eq!(params.max_tx_size, 16_384);
    assert_eq!(params.coins_per_utxo_byte, 4_310);
}

#[tokio::test]
async fn get_protocol_params_rejects_malformed_numeric_fields() {
    let url = spawn_protocol_params_mock(false).await;
    let provider = Provider::new(
        "blockfrost",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(url),
    )
    .expect("provider");

    let err = provider
        .get_protocol_params()
        .await
        .expect_err("malformed params must fail");
    assert!(format!("{err}").contains("invalid protocol param min_fee_a=oops"));
}
