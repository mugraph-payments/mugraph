use chrono::Utc;
use ed25519_dalek::{Signer, SigningKey};
use mugraph_core::types::{
    CrossNodeTransferRecord,
    TransferAckPayload,
    TransferAckStatus,
    TransferChainState,
    TransferCreditState,
    TransferInitPayload,
    TransferNoticePayload,
    TransferNoticeStage,
    TransferQueryType,
    TransferSettlementState,
    TransferStatusPayload,
    TransferStatusQueryPayload,
    XNodeAuth,
    XNodeEnvelope,
    XNodeMessageType,
};
use proptest::prelude::*;
use tempfile::TempDir;

use super::{
    audit::audit_status_events_with,
    auth::{
        AUTH_DOMAIN_SEP,
        MAX_CLOCK_SKEW_SECS,
        MAX_COMMAND_EXPIRY_HORIZON_SECS,
    },
    *,
};
use crate::{
    config::Config,
    database::{CROSS_NODE_TRANSFERS, Database, TRANSFER_AUDIT_LOG},
};

fn test_config_with_registry(path: &str) -> Config {
    Config::Server {
        addr: "127.0.0.1:9999".parse().unwrap(),
        seed: Some(42),
        secret_key: None,
        cardano_network: "preprod".to_string(),
        cardano_provider: "blockfrost".to_string(),
        cardano_api_key: None,
        cardano_provider_url: None,
        cardano_payment_sk: None,
        xnode_peer_registry_file: Some(path.to_string()),
        xnode_node_id: "node://b".to_string(),
        deposit_confirm_depth: 15,
        deposit_expiration_blocks: 1440,
        min_deposit_value: None,
        max_tx_size: 16384,
        max_withdrawal_fee: 2_000_000,
        fee_tolerance_pct: 5,
        dev_mode: false,
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

fn test_ctx(registry_path: &str) -> Context {
    let db_dir = TempDir::new().unwrap();
    let db_path = db_dir.path().join("db.redb");
    let database = std::sync::Arc::new(Database::setup(db_path).unwrap());
    database.migrate().unwrap();

    // Seed wallet for status response signing.
    let w = database.write().unwrap();
    {
        let mut t = w.open_table(crate::database::CARDANO_WALLET).unwrap();
        t.insert(
            "wallet",
            &mugraph_core::types::CardanoWallet::new(
                vec![7u8; 32],
                vec![8u8; 32],
                vec![],
                vec![],
                "addr_test...".to_string(),
                "preprod".to_string(),
            ),
        )
        .unwrap();
    }
    w.commit().unwrap();

    // Keep tempdir alive by leaking during test process lifetime.
    std::mem::forget(db_dir);

    let config = test_config_with_registry(registry_path);
    let keypair = config.keypair().unwrap();
    let registry =
        crate::peer_registry::PeerRegistry::load(registry_path).unwrap();

    Context {
        keypair,
        database,
        config,
        peer_registry: Some(std::sync::Arc::new(registry)),
    }
}

fn auth() -> XNodeAuth {
    XNodeAuth {
        alg: "Ed25519".to_string(),
        kid: "k1".to_string(),
        sig: String::new(),
    }
}

fn now_rfc3339_offset(secs: i64) -> String {
    (Utc::now() + chrono::Duration::seconds(secs)).to_rfc3339()
}

fn sign_envelope<T: serde::Serialize + Clone>(
    env: &mut XNodeEnvelope<T>,
    sk: &SigningKey,
) {
    let mut canonical = env.clone();
    canonical.auth.sig.clear();
    let body = serde_json::to_vec(&canonical).unwrap();
    let mut payload = Vec::with_capacity(AUTH_DOMAIN_SEP.len() + body.len());
    payload.extend_from_slice(AUTH_DOMAIN_SEP);
    payload.extend_from_slice(&body);
    let sig = sk.sign(&payload);
    env.auth.sig = muhex::encode(sig.to_bytes());
}

fn create_request() -> XNodeEnvelope<TransferInitPayload> {
    XNodeEnvelope {
        m: "xnode".to_string(),
        version: "3.0".to_string(),
        message_type: XNodeMessageType::TransferInit,
        message_id: "mid-1".to_string(),
        transfer_id: "tr-1".to_string(),
        idempotency_key: "ik-1".to_string(),
        correlation_id: "corr-1".to_string(),
        origin_node_id: "node://a".to_string(),
        destination_node_id: "node://b".to_string(),
        sent_at: now_rfc3339_offset(0),
        expires_at: Some(now_rfc3339_offset(120)),
        payload: TransferInitPayload {
            asset: "lovelace".to_string(),
            amount: "1".to_string(),
            destination_account_ref: "acct".to_string(),
            source_intent_hash: "hash".to_string(),
        },
        auth: auth(),
    }
}

fn seed_transfer(
    ctx: &Context,
    transfer_id: &str,
    chain_state: &str,
    credit_state: &str,
) {
    let w = ctx.database.write().unwrap();
    {
        let mut table = w.open_table(CROSS_NODE_TRANSFERS).unwrap();
        table
            .insert(
                transfer_id,
                &CrossNodeTransferRecord {
                    transfer_id: transfer_id.to_string(),
                    source_node_id: "node://a".to_string(),
                    destination_node_id: "node://b".to_string(),
                    tx_hash: Some("txhash".to_string()),
                    chain_state: chain_state.to_string(),
                    credit_state: credit_state.to_string(),
                    confirmations_observed: 7,
                    created_at: 1,
                    updated_at: 2,
                },
            )
            .unwrap();
    }
    w.commit().unwrap();
}

#[test]
fn receive_metric_name_is_not_send_metric() {
    assert_eq!(M3_MESSAGE_RECEIVE_COUNTER, "mugraph_message_receive_total");
}

#[test]
fn create_rejects_invalid_signature_and_records_audit_event() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);

    let mut request = create_request();
    request.auth.sig = "deadbeef".to_string();

    let err = handle_create(&request, &ctx).unwrap_err();
    assert!(matches!(err, Error::InvalidInput { .. }));

    let read = ctx.database.read().unwrap();
    let table = read.open_table(TRANSFER_AUDIT_LOG).unwrap();
    let mut count = 0usize;
    for item in table.iter().unwrap() {
        let (_, value) = item.unwrap();
        let evt = value.value();
        if evt.transfer_id == request.transfer_id {
            count += 1;
        }
    }
    assert!(count >= 1);
}

#[test]
fn requests_are_rejected_if_peer_registry_file_disappears() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);

    std::fs::remove_file(&registry_path).unwrap();

    let mut request = create_request();
    sign_envelope(&mut request, &signer);

    let err = handle_create(&request, &ctx).unwrap_err();
    match err {
        Error::InvalidInput { reason } => {
            assert!(
                reason.contains("AUTHZ_DENIED")
                    || reason.contains("failed to read peer registry")
            );
        }
        _ => panic!("expected InvalidInput authz error"),
    }
}

#[test]
fn runtime_registry_reload_honors_revocation() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);

    let mut request = create_request();
    sign_envelope(&mut request, &signer);
    assert!(validate_auth_signature(&request, &ctx).is_ok());

    let revoked_json = format!(
        r#"{{"peers":[{{"node_id":"node://a","endpoint":"https://a.example/rpc","auth_alg":"Ed25519","kid":"k1","public_key_hex":"{}","revoked":true}}]}}"#,
        muhex::encode(signer.verifying_key().to_bytes())
    );
    std::fs::write(&registry_path, revoked_json).unwrap();

    let err = validate_auth_signature(&request, &ctx).unwrap_err();
    match err {
        Error::InvalidInput { reason } => {
            assert!(reason.contains("UNKNOWN_KEY_ID"));
        }
        _ => panic!("expected invalid input with unknown key id"),
    }
}

#[test]
fn duplicate_message_id_is_rejected_deterministically() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);

    let mut request = create_request();
    sign_envelope(&mut request, &signer);

    let first = handle_create(&request, &ctx).unwrap();
    assert!(matches!(
        first,
        Response::CrossNodeTransferCreate { accepted: true, .. }
    ));

    let second = handle_create(&request, &ctx).unwrap_err();
    match second {
        Error::InvalidInput { reason } => {
            assert!(reason.contains("REPLAY_DETECTED"));
        }
        _ => panic!("expected InvalidInput replay error"),
    }
}

#[test]
fn idempotency_conflict_is_rejected() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);

    let mut request_a = create_request();
    sign_envelope(&mut request_a, &signer);
    handle_create(&request_a, &ctx).unwrap();

    let mut request_b = create_request();
    request_b.message_id = "mid-2".to_string();
    request_b.payload.amount = "2".to_string();
    sign_envelope(&mut request_b, &signer);

    let err = handle_create(&request_b, &ctx).unwrap_err();
    match err {
        Error::InvalidInput { reason } => {
            assert!(reason.contains("IDEMPOTENCY_CONFLICT"))
        }
        _ => panic!("expected InvalidInput conflict"),
    }
}

#[test]
fn create_rejects_duplicate_transfer_id_across_idempotency_keys() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);

    let mut first = create_request();
    sign_envelope(&mut first, &signer);
    handle_create(&first, &ctx).unwrap();

    let mut second = create_request();
    second.message_id = "mid-2".to_string();
    second.idempotency_key = "ik-2".to_string();
    second.payload.destination_account_ref = "acct-2".to_string();
    sign_envelope(&mut second, &signer);

    let err = handle_create(&second, &ctx).unwrap_err();
    match err {
        Error::InvalidInput { reason } => {
            assert!(reason.contains("TRANSFER_ALREADY_EXISTS"))
        }
        _ => panic!("expected duplicate transfer_id rejection"),
    }
}

#[test]
fn create_rejects_destination_binding_mismatch() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);

    let mut request = create_request();
    request.destination_node_id = "node://other".to_string();
    sign_envelope(&mut request, &signer);

    let err = handle_create(&request, &ctx).unwrap_err();
    match err {
        Error::InvalidInput { reason } => {
            assert!(reason.contains("AUTHZ_DENIED"))
        }
        _ => panic!("expected InvalidInput authz error"),
    }
}

#[test]
fn create_rejects_sent_at_outside_clock_skew_window() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);

    let mut request = create_request();
    request.sent_at = now_rfc3339_offset(-MAX_CLOCK_SKEW_SECS - 30);
    request.expires_at = Some(now_rfc3339_offset(60));
    sign_envelope(&mut request, &signer);

    let err = handle_create(&request, &ctx).unwrap_err();
    match err {
        Error::InvalidInput { reason } => {
            assert!(reason.contains("REPLAY_DETECTED"))
        }
        _ => panic!("expected InvalidInput replay error"),
    }
}

#[test]
fn create_rejects_expiry_horizon_above_policy() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);

    let mut request = create_request();
    request.sent_at = now_rfc3339_offset(0);
    request.expires_at =
        Some(now_rfc3339_offset(MAX_COMMAND_EXPIRY_HORIZON_SECS + 60));
    sign_envelope(&mut request, &signer);

    let err = handle_create(&request, &ctx).unwrap_err();
    match err {
        Error::InvalidInput { reason } => {
            assert!(reason.contains("SCHEMA_VALIDATION_FAILED"))
        }
        _ => panic!("expected InvalidInput schema error"),
    }
}

#[test]
fn status_response_contains_chain_and_credit_state() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);
    seed_transfer(&ctx, "tr", "submitted", "none");

    let mut request = XNodeEnvelope {
        m: "xnode".to_string(),
        version: "3.0".to_string(),
        message_type: XNodeMessageType::TransferStatusQuery,
        message_id: "mid".to_string(),
        transfer_id: "tr".to_string(),
        idempotency_key: "ik".to_string(),
        correlation_id: "corr".to_string(),
        origin_node_id: "node://a".to_string(),
        destination_node_id: "node://b".to_string(),
        sent_at: now_rfc3339_offset(0),
        expires_at: None,
        payload: TransferStatusQueryPayload {
            query_type: TransferQueryType::Current,
        },
        auth: auth(),
    };
    sign_envelope(&mut request, &signer);

    let response = handle_status(&request, &ctx).unwrap();
    match response {
        Response::CrossNodeTransferStatus(env) => {
            assert_eq!(env.payload.source_state, "submitted");
            assert_eq!(env.payload.destination_state, "chain_observed");
            assert_eq!(env.payload.chain_state, TransferChainState::Submitted);
            assert_eq!(env.payload.credit_state, TransferCreditState::None);
        }
        _ => panic!("unexpected response variant"),
    }
}

#[test]
fn status_mapping_invalidated_held_maps_to_manual_review() {
    let record = CrossNodeTransferRecord {
        transfer_id: "tr".to_string(),
        source_node_id: "node://a".to_string(),
        destination_node_id: "node://b".to_string(),
        tx_hash: Some("h".to_string()),
        chain_state: "invalidated".to_string(),
        credit_state: "held".to_string(),
        confirmations_observed: 3,
        created_at: 1,
        updated_at: 2,
    };

    let payload = status_payload_from_record(&record);
    assert_eq!(payload.source_state, "invalidated");
    assert_eq!(payload.destination_state, "invalidated");
    assert_eq!(
        payload.settlement_state,
        TransferSettlementState::ManualReview
    );
    assert_eq!(payload.chain_state, TransferChainState::Invalidated);
    assert_eq!(payload.credit_state, TransferCreditState::Held);
}

#[test]
fn status_query_emits_required_audit_events() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);

    seed_transfer(&ctx, "tr-status", "confirmed", "credited");

    let mut request = XNodeEnvelope {
        m: "xnode".to_string(),
        version: "3.0".to_string(),
        message_type: XNodeMessageType::TransferStatusQuery,
        message_id: "mid-status".to_string(),
        transfer_id: "tr-status".to_string(),
        idempotency_key: "ik-status".to_string(),
        correlation_id: "corr".to_string(),
        origin_node_id: "node://a".to_string(),
        destination_node_id: "node://b".to_string(),
        sent_at: now_rfc3339_offset(0),
        expires_at: None,
        payload: TransferStatusQueryPayload {
            query_type: TransferQueryType::Current,
        },
        auth: auth(),
    };
    sign_envelope(&mut request, &signer);

    let _ = handle_status(&request, &ctx).unwrap();

    let read = ctx.database.read().unwrap();
    let table = read.open_table(TRANSFER_AUDIT_LOG).unwrap();
    let mut confirmed = false;
    let mut credited = false;
    for row in table.iter().unwrap() {
        let (_k, v) = row.unwrap();
        let evt = v.value();
        if evt.transfer_id == "tr-status"
            && evt.event_type == "transfer.confirmed"
        {
            confirmed = true;
        }
        if evt.transfer_id == "tr-status"
            && evt.event_type == "transfer.credited"
        {
            credited = true;
        }
    }

    assert!(confirmed);
    assert!(credited);
}

#[test]
fn audit_status_events_propagates_audit_failures() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);

    let request = XNodeEnvelope {
        m: "xnode".to_string(),
        version: "3.0".to_string(),
        message_type: XNodeMessageType::TransferStatusQuery,
        message_id: "mid-status-fail".to_string(),
        transfer_id: "tr-status-fail".to_string(),
        idempotency_key: "ik-status-fail".to_string(),
        correlation_id: "corr".to_string(),
        origin_node_id: "node://a".to_string(),
        destination_node_id: "node://b".to_string(),
        sent_at: now_rfc3339_offset(0),
        expires_at: None,
        payload: TransferStatusQueryPayload {
            query_type: TransferQueryType::Current,
        },
        auth: auth(),
    };

    let payload = TransferStatusPayload {
        source_state: "confirmed".to_string(),
        destination_state: "credited".to_string(),
        settlement_state: TransferSettlementState::Confirmed,
        chain_state: TransferChainState::Confirmed,
        credit_state: TransferCreditState::Credited,
        tx_hash: Some("h".to_string()),
        confirmations_observed: 7,
        updated_at: "1".to_string(),
    };

    let err = audit_status_events_with(
        &ctx,
        &request,
        &payload,
        |_event_type, _reason| {
            Err(Error::Internal {
                reason: "audit failed".to_string(),
            })
        },
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("audit failed"));
}

#[test]
fn ack_accepts_valid_request() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);

    let mut request = XNodeEnvelope {
        m: "xnode".to_string(),
        version: "3.0".to_string(),
        message_type: XNodeMessageType::TransferAck,
        message_id: "mid".to_string(),
        transfer_id: "tr".to_string(),
        idempotency_key: "ik".to_string(),
        correlation_id: "corr".to_string(),
        origin_node_id: "node://a".to_string(),
        destination_node_id: "node://b".to_string(),
        sent_at: now_rfc3339_offset(0),
        expires_at: Some(now_rfc3339_offset(120)),
        payload: TransferAckPayload {
            ack_for_message_id: "mid2".to_string(),
            ack_status: TransferAckStatus::Processed,
            ack_at: "2026-02-26T18:00:01Z".to_string(),
        },
        auth: auth(),
    };
    sign_envelope(&mut request, &signer);

    let response = handle_ack(&request, &ctx).unwrap();
    assert!(matches!(
        response,
        Response::CrossNodeTransferAck { accepted: true }
    ));
}

#[test]
fn notify_rejects_unknown_transfer_id() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);

    let mut request = XNodeEnvelope {
        m: "xnode".to_string(),
        version: "3.0".to_string(),
        message_type: XNodeMessageType::TransferNotice,
        message_id: "mid-notice".to_string(),
        transfer_id: "tr-missing".to_string(),
        idempotency_key: "ik-notice".to_string(),
        correlation_id: "corr-notice".to_string(),
        origin_node_id: "node://a".to_string(),
        destination_node_id: "node://b".to_string(),
        sent_at: now_rfc3339_offset(0),
        expires_at: Some(now_rfc3339_offset(120)),
        payload: TransferNoticePayload {
            notice_stage: TransferNoticeStage::Confirmed,
            tx_hash: "abcd".to_string(),
            confirmations: Some(6),
        },
        auth: auth(),
    };
    sign_envelope(&mut request, &signer);

    let err = handle_notify(&request, &ctx).unwrap_err();
    match err {
        Error::InvalidInput { reason } => {
            assert!(reason.contains("TRANSFER_NOT_FOUND"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn notify_persists_notice_derived_chain_state() {
    let signer = SigningKey::from_bytes(&[7u8; 32]);
    let registry_dir = TempDir::new().unwrap();
    let registry_path = write_registry(&registry_dir, &signer);
    let ctx = test_ctx(&registry_path);

    let mut create = create_request();
    sign_envelope(&mut create, &signer);
    handle_create(&create, &ctx).unwrap();

    let mut notify = XNodeEnvelope {
        m: "xnode".to_string(),
        version: "3.0".to_string(),
        message_type: XNodeMessageType::TransferNotice,
        message_id: "mid-notice-ok".to_string(),
        transfer_id: create.transfer_id.clone(),
        idempotency_key: "ik-notice-ok".to_string(),
        correlation_id: create.correlation_id.clone(),
        origin_node_id: "node://a".to_string(),
        destination_node_id: "node://b".to_string(),
        sent_at: now_rfc3339_offset(0),
        expires_at: Some(now_rfc3339_offset(120)),
        payload: TransferNoticePayload {
            notice_stage: TransferNoticeStage::Confirmed,
            tx_hash: "abcd".to_string(),
            confirmations: Some(6),
        },
        auth: auth(),
    };
    sign_envelope(&mut notify, &signer);

    handle_notify(&notify, &ctx).unwrap();

    let read = ctx.database.read().unwrap();
    let table = read.open_table(CROSS_NODE_TRANSFERS).unwrap();
    let record = table
        .get(create.transfer_id.as_str())
        .unwrap()
        .unwrap()
        .value();
    assert_eq!(record.tx_hash.as_deref(), Some("abcd"));
    assert_eq!(record.confirmations_observed, 6);
    assert_ne!(record.chain_state, "unknown");
}

fn chain_rank(chain: &TransferChainState) -> u8 {
    match chain {
        TransferChainState::Unknown => 0,
        TransferChainState::Submitted => 1,
        TransferChainState::Confirming => 2,
        TransferChainState::Confirmed => 3,
        TransferChainState::Invalidated => 4,
    }
}

fn settlement_rank(settlement: &TransferSettlementState) -> u8 {
    match settlement {
        TransferSettlementState::NotSubmitted => 0,
        TransferSettlementState::Submitted => 1,
        TransferSettlementState::Confirming => 2,
        TransferSettlementState::Confirmed => 3,
        TransferSettlementState::Invalidated => 4,
        TransferSettlementState::ManualReview => 5,
    }
}

proptest! {
    #[test]
    fn prop_same_idempotency_same_payload_is_side_effect_free_on_accept(
        amount in 1u64..=1_000_000,
    ) {
        let signer = SigningKey::from_bytes(&[7u8; 32]);
        let registry_dir = TempDir::new().unwrap();
        let registry_path = write_registry(&registry_dir, &signer);
        let ctx = test_ctx(&registry_path);

        let mut first = create_request();
        first.payload.amount = amount.to_string();
        first.message_id = "mid-1".to_string();
        first.idempotency_key = "ik-1".to_string();
        sign_envelope(&mut first, &signer);

        let mut second = first.clone();
        second.message_id = "mid-2".to_string();
        sign_envelope(&mut second, &signer);

        let a = handle_create(&first, &ctx).unwrap();
        let b = handle_create(&second, &ctx).unwrap();

        let a_ok = matches!(a, Response::CrossNodeTransferCreate { accepted: true, .. });
        let b_ok = matches!(b, Response::CrossNodeTransferCreate { accepted: true, .. });
        prop_assert!(a_ok && b_ok);
    }

    #[test]
    fn prop_expired_command_is_rejected(
        seconds in 0u8..=59,
    ) {
        let signer = SigningKey::from_bytes(&[7u8; 32]);
        let registry_dir = TempDir::new().unwrap();
        let registry_path = write_registry(&registry_dir, &signer);
        let ctx = test_ctx(&registry_path);

        let mut request = create_request();
        request.sent_at = format!("2026-02-26T18:00:{seconds:02}Z");
        request.expires_at = Some(format!("2026-02-26T18:00:{seconds:02}Z"));
        sign_envelope(&mut request, &signer);

        let err = handle_create(&request, &ctx).unwrap_err();
        let is_invalid = matches!(err, Error::InvalidInput { .. });
        prop_assert!(is_invalid);
    }

    #[test]
    fn prop_status_mapping_is_monotonic_except_invalidation(
        credit in prop_oneof![
            Just("none".to_string()),
            Just("eligible".to_string()),
            Just("credited".to_string()),
        ],
    ) {
        let states = ["unknown", "submitted", "confirming", "confirmed"];
        let mut prev_chain = 0u8;
        let mut prev_settlement = 0u8;

        for state in states {
            let record = CrossNodeTransferRecord {
                transfer_id: "tr".to_string(),
                source_node_id: "node://a".to_string(),
                destination_node_id: "node://b".to_string(),
                tx_hash: Some("h".to_string()),
                chain_state: state.to_string(),
                credit_state: credit.clone(),
                confirmations_observed: 1,
                created_at: 1,
                updated_at: 2,
            };

            let payload = status_payload_from_record(&record);
            let c = chain_rank(&payload.chain_state);
            let s = settlement_rank(&payload.settlement_state);
            prop_assert!(c >= prev_chain);
            prop_assert!(s >= prev_settlement);
            prev_chain = c;
            prev_settlement = s;
        }

        let invalidated = CrossNodeTransferRecord {
            transfer_id: "tr".to_string(),
            source_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            tx_hash: Some("h".to_string()),
            chain_state: "invalidated".to_string(),
            credit_state: "held".to_string(),
            confirmations_observed: 1,
            created_at: 1,
            updated_at: 2,
        };
        let payload = status_payload_from_record(&invalidated);
        prop_assert_eq!(payload.settlement_state, TransferSettlementState::ManualReview);
    }

    #[test]
    fn prop_notify_rejects_any_non_notice_message_type(mt in 0u8..=3) {
        let signer = SigningKey::from_bytes(&[7u8; 32]);
        let registry_dir = TempDir::new().unwrap();
        let registry_path = write_registry(&registry_dir, &signer);
        let ctx = test_ctx(&registry_path);

        let message_type = match mt {
            0 => XNodeMessageType::TransferInit,
            1 => XNodeMessageType::TransferStatusQuery,
            2 => XNodeMessageType::TransferStatus,
            _ => XNodeMessageType::TransferAck,
        };

        let mut request = XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.0".to_string(),
            message_type,
            message_id: "mid".to_string(),
            transfer_id: "tr".to_string(),
            idempotency_key: "ik".to_string(),
            correlation_id: "corr".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: "2026-02-26T18:00:00Z".to_string(),
            expires_at: Some("2026-02-26T18:05:00Z".to_string()),
            payload: TransferNoticePayload {
                notice_stage: TransferNoticeStage::Confirmed,
                tx_hash: "abcd".to_string(),
                confirmations: Some(1),
            },
            auth: auth(),
        };
        sign_envelope(&mut request, &signer);

        let err = handle_notify(&request, &ctx).unwrap_err();
        let is_unsupported = matches!(err, Error::UnsupportedMessageType { .. });
        prop_assert!(is_unsupported);
    }
}
