use std::{future::Future, path::Path, sync::OnceLock};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request as HttpRequest, StatusCode},
};
use ed25519_dalek::{Signer, SigningKey};
use mugraph_core::types::{
    CardanoWallet,
    Request,
    Response,
    TransferAckPayload,
    TransferAckStatus,
    TransferInitPayload,
    TransferNoticePayload,
    TransferNoticeStage,
    TransferQueryType,
    TransferStatusQueryPayload,
    XNodeAuth,
    XNodeEnvelope,
    XNodeMessageType,
};
use mugraph_node::{
    config::Config,
    database::{CARDANO_WALLET, Database},
    routes::router,
};
use tempfile::TempDir;
use tower::util::ServiceExt;

fn env_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

async fn with_db_path<T, Fut>(path: &Path, f: impl FnOnce() -> Fut) -> T
where
    Fut: Future<Output = T>,
{
    let _guard = env_lock().lock().await;
    let previous = std::env::var_os("MUGRAPH_DB_PATH");
    unsafe {
        std::env::set_var("MUGRAPH_DB_PATH", path);
    }
    let result = f().await;
    match previous {
        Some(value) => unsafe { std::env::set_var("MUGRAPH_DB_PATH", value) },
        None => unsafe { std::env::remove_var("MUGRAPH_DB_PATH") },
    }
    result
}

fn test_config(peer_registry_file: String) -> Config {
    Config::Server {
        addr: "127.0.0.1:9999".parse().unwrap(),
        seed: Some(42),
        secret_key: None,
        cardano_network: "preprod".to_string(),
        cardano_provider: "blockfrost".to_string(),
        cardano_api_key: None,
        cardano_provider_url: None,
        cardano_payment_sk: None,
        xnode_peer_registry_file: Some(peer_registry_file),
        xnode_node_id: "node://b".to_string(),
        deposit_confirm_depth: 15,
        deposit_expiration_blocks: 1440,
        min_deposit_value: Some(1_000_000),
        max_tx_size: 16_384,
        max_withdrawal_fee: 2_000_000,
        fee_tolerance_pct: 5,
        dev_mode: true,
    }
}

fn write_registry(dir: &TempDir, pk: &SigningKey) -> String {
    let path = dir.path().join("peers.json");
    let json = format!(
        r#"{{"peers":[{{"node_id":"node://a","endpoint":"https://a.example/rpc","auth_alg":"Ed25519","kid":"k1","public_key_hex":"{}","revoked":false}}]}}"#,
        muhex::encode(pk.verifying_key().to_bytes())
    );
    std::fs::write(&path, json).unwrap();
    path.display().to_string()
}

fn seed_wallet(path: &Path) {
    let db = Database::setup(path).unwrap();
    db.migrate().unwrap();

    let w = db.write().unwrap();
    {
        let mut t = w.open_table(CARDANO_WALLET).unwrap();
        t.insert(
            "wallet",
            &CardanoWallet::new(
                vec![7u8; 32],
                vec![8u8; 32],
                vec![],
                vec![],
                "addr_test1script".to_string(),
                "preprod".to_string(),
            ),
        )
        .unwrap();
    }
    w.commit().unwrap();
}

fn now_rfc3339_offset(secs: i64) -> String {
    (chrono::Utc::now() + chrono::Duration::seconds(secs)).to_rfc3339()
}

fn sign_envelope<T: serde::Serialize + Clone>(
    mut env: XNodeEnvelope<T>,
    sk: &SigningKey,
) -> XNodeEnvelope<T> {
    let mut canonical = env.clone();
    canonical.auth.sig.clear();
    let body = serde_json::to_vec(&canonical).unwrap();
    let mut payload =
        Vec::with_capacity("mugraph_xnode_auth_v1".len() + body.len());
    payload.extend_from_slice(b"mugraph_xnode_auth_v1");
    payload.extend_from_slice(&body);
    env.auth.sig = muhex::encode(sk.sign(&payload).to_bytes());
    env
}

async fn send_rpc(app: &Router, request: Request) -> Response {
    let response = app
        .clone()
        .oneshot(
            HttpRequest::post("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test(flavor = "current_thread")]
async fn cross_node_rpc_smoke_preserves_create_notify_status_ack_flow() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("cross-node-smoke.redb");
    seed_wallet(&db_path);

    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_path = write_registry(&dir, &signer);

    let app = with_db_path(&db_path, || async {
        let config = test_config(registry_path);
        let keypair = config.keypair().unwrap();
        router(config, keypair).await.unwrap()
    })
    .await;

    let create = Request::CrossNodeTransferCreate(sign_envelope(
        XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferInit,
            message_id: "mid-create".to_string(),
            transfer_id: "tr-smoke".to_string(),
            idempotency_key: "ik-create".to_string(),
            correlation_id: "corr-1".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: now_rfc3339_offset(0),
            expires_at: Some(now_rfc3339_offset(120)),
            payload: TransferInitPayload {
                asset: "lovelace".to_string(),
                amount: "1000000".to_string(),
                destination_account_ref: "acct-1".to_string(),
                source_intent_hash: "ab".repeat(32),
            },
            auth: XNodeAuth {
                alg: "Ed25519".to_string(),
                kid: "k1".to_string(),
                sig: String::new(),
            },
        },
        &signer,
    ));

    let created = send_rpc(&app, create).await;
    assert!(matches!(
        created,
        Response::CrossNodeTransferCreate {
            accepted: true,
            ref transfer_id,
        } if transfer_id == "tr-smoke"
    ));

    let notice_tx_hash = "cd".repeat(32);
    let notify = Request::CrossNodeTransferNotify(sign_envelope(
        XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferNotice,
            message_id: "mid-notify".to_string(),
            transfer_id: "tr-smoke".to_string(),
            idempotency_key: "ik-notify".to_string(),
            correlation_id: "corr-1".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: now_rfc3339_offset(0),
            expires_at: Some(now_rfc3339_offset(120)),
            payload: TransferNoticePayload {
                notice_stage: TransferNoticeStage::Submitted,
                tx_hash: notice_tx_hash.clone(),
                confirmations: Some(1),
            },
            auth: XNodeAuth {
                alg: "Ed25519".to_string(),
                kid: "k1".to_string(),
                sig: String::new(),
            },
        },
        &signer,
    ));

    let notified = send_rpc(&app, notify).await;
    assert!(matches!(
        notified,
        Response::CrossNodeTransferNotify { accepted: true }
    ));

    let status = Request::CrossNodeTransferStatus(sign_envelope(
        XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferStatusQuery,
            message_id: "mid-status".to_string(),
            transfer_id: "tr-smoke".to_string(),
            idempotency_key: "ik-status".to_string(),
            correlation_id: "corr-1".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: now_rfc3339_offset(0),
            expires_at: None,
            payload: TransferStatusQueryPayload {
                query_type: TransferQueryType::Current,
            },
            auth: XNodeAuth {
                alg: "Ed25519".to_string(),
                kid: "k1".to_string(),
                sig: String::new(),
            },
        },
        &signer,
    ));

    let status_response = send_rpc(&app, status).await;
    match status_response {
        Response::CrossNodeTransferStatus(env) => {
            assert_eq!(env.transfer_id, "tr-smoke");
            assert_eq!(
                env.payload.chain_state,
                mugraph_core::types::TransferChainState::Submitted
            );
            assert_eq!(
                env.payload.tx_hash.as_deref(),
                Some(notice_tx_hash.as_str())
            );
        }
        other => panic!("unexpected status response: {other:?}"),
    }

    let ack = Request::CrossNodeTransferAck(sign_envelope(
        XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferAck,
            message_id: "mid-ack".to_string(),
            transfer_id: "tr-smoke".to_string(),
            idempotency_key: "ik-ack".to_string(),
            correlation_id: "corr-1".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: now_rfc3339_offset(0),
            expires_at: Some(now_rfc3339_offset(120)),
            payload: TransferAckPayload {
                ack_for_message_id: "mid-notify".to_string(),
                ack_status: TransferAckStatus::Processed,
                ack_at: now_rfc3339_offset(0),
            },
            auth: XNodeAuth {
                alg: "Ed25519".to_string(),
                kid: "k1".to_string(),
                sig: String::new(),
            },
        },
        &signer,
    ));

    let acknowledged = send_rpc(&app, ack).await;
    assert!(matches!(
        acknowledged,
        Response::CrossNodeTransferAck { accepted: true }
    ));
}
