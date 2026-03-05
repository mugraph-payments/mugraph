use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use mugraph_node::provider::Provider;
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
