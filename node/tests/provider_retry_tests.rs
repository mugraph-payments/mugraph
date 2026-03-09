use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use axum::{
    Router,
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use mugraph_node::provider::{Provider, TxSettlementState};
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

async fn spawn_transport_error_mock(expected_attempts: usize) -> (String, Arc<AtomicUsize>) {
    let hits = Arc::new(AtomicUsize::new(0));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind transport error server");
    let addr = listener.local_addr().expect("local addr");
    let hits_clone = hits.clone();

    tokio::spawn(async move {
        while hits_clone.load(Ordering::SeqCst) < expected_attempts {
            let (stream, _) = listener.accept().await.expect("accept connection");
            hits_clone.fetch_add(1, Ordering::SeqCst);
            drop(stream);
        }
    });

    (format!("http://{}", addr), hits)
}

async fn spawn_observation_mock(
    tip_height: u64,
    tx_block_height: Option<u64>,
) -> String {
    async fn latest_block_with_height(
        State((height, _tx_block_height)): State<(u64, Option<u64>)>,
    ) -> impl IntoResponse {
        (
            StatusCode::OK,
            axum::Json(json!({"slot": 42, "hash": "abc", "height": height})),
        )
    }

    async fn tx_info_with_height(
        State((_height, tx_block_height)): State<(u64, Option<u64>)>,
    ) -> impl IntoResponse {
        match tx_block_height {
            Some(height) => (StatusCode::OK, axum::Json(json!({"block_height": height}))).into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }

    let app = Router::new()
        .route("/blocks/latest", get(latest_block_with_height))
        .route("/txs/{tx_hash}", get(tx_info_with_height))
        .with_state((tip_height, tx_block_height));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://{addr}")
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

fn assert_maestro_api_key(headers: &HeaderMap) {
    assert_eq!(headers.get("api-key").and_then(|v| v.to_str().ok()), Some("test-key"));
}

async fn maestro_latest_block(headers: HeaderMap) -> impl IntoResponse {
    assert_maestro_api_key(&headers);
    (StatusCode::OK, axum::Json(json!({"slot": 77, "hash": "def", "height": 20})))
}

async fn maestro_protocol_params_ok(headers: HeaderMap) -> impl IntoResponse {
    assert_maestro_api_key(&headers);
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
        "coins_per_utxo_byte": "4310"
    }))
}

async fn maestro_protocol_params_bad(headers: HeaderMap) -> impl IntoResponse {
    assert_maestro_api_key(&headers);
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
        "coins_per_utxo_byte": "4310"
    }))
}

async fn maestro_submit_ok(headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    assert_maestro_api_key(&headers);
    assert!(!body.is_empty(), "submitted tx body should not be empty");
    (StatusCode::OK, axum::Json(json!({"hash": "cd".repeat(32)})))
}

async fn maestro_submit_bad(headers: HeaderMap) -> impl IntoResponse {
    assert_maestro_api_key(&headers);
    (StatusCode::BAD_REQUEST, "bad tx body")
}

async fn maestro_output(Path((tx_hash, index)): Path<(String, u16)>, headers: HeaderMap) -> impl IntoResponse {
    assert_maestro_api_key(&headers);
    if tx_hash.starts_with("ff") {
        return StatusCode::NOT_FOUND.into_response();
    }

    (
        StatusCode::OK,
        axum::Json(json!({
            "address": "addr_test1maestro",
            "assets": [{"unit": "lovelace", "quantity": "4000000"}],
            "datum_hash": null,
            "datum": null,
            "reference_script_hash": null,
            "block_height": 123,
            "index_echo": index
        })),
    )
        .into_response()
}

async fn maestro_tx_info(Path(tx_hash): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    assert_maestro_api_key(&headers);
    if tx_hash.starts_with("ff") {
        return StatusCode::NOT_FOUND.into_response();
    }

    (StatusCode::OK, axum::Json(json!({"block_height": 123}))).into_response()
}

async fn maestro_address_utxos(
    Path(_address): Path<String>,
    Query(query): Query<PaginationQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    assert_maestro_api_key(&headers);
    let page = query.page.unwrap_or(1);
    match page {
        1 => axum::Json(json!([
            {
                "tx_hash": "44".repeat(32),
                "tx_index": 0,
                "assets": [{"unit": "lovelace", "quantity": "2000000"}],
                "datum_hash": null,
                "reference_script_hash": null
            },
            {
                "tx_hash": "55".repeat(32),
                "tx_index": 1,
                "assets": [{"unit": "lovelace", "quantity": "3000000"}],
                "datum_hash": "abcd",
                "reference_script_hash": null
            }
        ]))
        .into_response(),
        _ => axum::Json(json!([])).into_response(),
    }
}

async fn spawn_maestro_mock(valid_params: bool, submit_ok_status: bool) -> String {
    let app = Router::new()
        .route("/blocks/latest", get(maestro_latest_block))
        .route(
            "/protocol-params",
            if valid_params {
                get(maestro_protocol_params_ok)
            } else {
                get(maestro_protocol_params_bad)
            },
        )
        .route(
            "/transactions",
            if submit_ok_status {
                post(maestro_submit_ok)
            } else {
                post(maestro_submit_bad)
            },
        )
        .route("/transactions/{tx_hash}", get(maestro_tx_info))
        .route("/transactions/{tx_hash}/outputs/{index}", get(maestro_output))
        .route("/addresses/{address}/utxos", get(maestro_address_utxos));

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
async fn get_tip_retries_transport_errors_and_reports_network_failure() {
    let (url, hits) = spawn_transport_error_mock(3).await;

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
        .expect_err("transport failures must fail after retries");
    let msg = format!("{err}");
    assert!(msg.contains("network error after 3 attempts"));
    assert!(!msg.contains("status"));
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

#[tokio::test]
async fn maestro_get_tip_uses_api_key_header_and_parses_response() {
    let url = spawn_maestro_mock(true, true).await;
    let provider = Provider::new(
        "maestro",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(url),
    )
    .expect("provider");

    let tip = provider.get_tip().await.expect("tip");
    assert_eq!(tip.block_height, 20);
    assert_eq!(tip.slot, 77);
    assert_eq!(tip.hash, "def");
}

#[tokio::test]
async fn maestro_submit_tx_parses_hash_response() {
    let url = spawn_maestro_mock(true, true).await;
    let provider = Provider::new(
        "maestro",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(url),
    )
    .expect("provider");

    let response = provider
        .submit_tx(&[0xde, 0xad, 0xbe, 0xef])
        .await
        .expect("submit ok");
    assert_eq!(response.tx_hash, "cd".repeat(32));
}

#[tokio::test]
async fn maestro_get_protocol_params_parses_numeric_fields() {
    let url = spawn_maestro_mock(true, true).await;
    let provider = Provider::new(
        "maestro",
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
async fn maestro_get_protocol_params_rejects_malformed_numeric_fields() {
    let url = spawn_maestro_mock(false, true).await;
    let provider = Provider::new(
        "maestro",
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

#[tokio::test]
async fn maestro_get_utxo_maps_found_and_missing_outputs() {
    let url = spawn_maestro_mock(true, true).await;
    let provider = Provider::new(
        "maestro",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(url),
    )
    .expect("provider");

    let utxo = provider
        .get_utxo(&"11".repeat(32), 0)
        .await
        .expect("get utxo")
        .expect("utxo should exist");
    assert_eq!(utxo.address, "addr_test1maestro");
    assert_eq!(utxo.output_index, 0);
    assert_eq!(utxo.block_height, Some(123));
    assert_eq!(utxo.amount[0].quantity, "4000000");

    let missing = provider
        .get_utxo(&"ff".repeat(32), 0)
        .await
        .expect("missing lookup should succeed");
    assert!(missing.is_none());
}

#[tokio::test]
async fn maestro_get_address_utxos_paginates_and_maps_assets() {
    let url = spawn_maestro_mock(true, true).await;
    let provider = Provider::new(
        "maestro",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(url),
    )
    .expect("provider");

    let utxos = provider
        .get_address_utxos("addr_test1maestro")
        .await
        .expect("address utxos");

    assert_eq!(utxos.len(), 2);
    assert_eq!(utxos[0].tx_hash, "44".repeat(32));
    assert_eq!(utxos[0].output_index, 0);
    assert_eq!(utxos[0].block_height, None);
    assert_eq!(utxos[1].datum_hash.as_deref(), Some("abcd"));
}

#[tokio::test]
async fn observe_tx_status_combines_tip_and_tx_height_consistently() {
    let confirming_url = spawn_observation_mock(100, Some(90)).await;
    let confirming_provider = Provider::new(
        "blockfrost",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(confirming_url),
    )
    .expect("provider");
    let confirming = confirming_provider
        .observe_tx_status(&"11".repeat(32), 12, 3, false)
        .await
        .expect("confirming observation");
    assert_eq!(confirming.tx_block_height, Some(90));
    assert_eq!(confirming.tip_height, 100);
    assert_eq!(confirming.confirmations, 11);
    assert_eq!(confirming.state, TxSettlementState::Confirming);

    let confirmed_url = spawn_observation_mock(101, Some(90)).await;
    let confirmed_provider = Provider::new(
        "blockfrost",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(confirmed_url),
    )
    .expect("provider");
    let confirmed = confirmed_provider
        .observe_tx_status(&"22".repeat(32), 12, 3, false)
        .await
        .expect("confirmed observation");
    assert_eq!(confirmed.confirmations, 12);
    assert_eq!(confirmed.state, TxSettlementState::Confirmed);

    let invalidated_url = spawn_observation_mock(101, None).await;
    let invalidated_provider = Provider::new(
        "blockfrost",
        "test-key".to_string(),
        "preprod".to_string(),
        Some(invalidated_url),
    )
    .expect("provider");
    let invalidated = invalidated_provider
        .observe_tx_status(&"33".repeat(32), 12, 3, true)
        .await
        .expect("invalidated observation");
    assert_eq!(invalidated.tx_block_height, None);
    assert_eq!(invalidated.confirmations, 0);
    assert_eq!(invalidated.state, TxSettlementState::Invalidated);
}
