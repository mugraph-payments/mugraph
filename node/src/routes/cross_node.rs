use std::time::{SystemTime, UNIX_EPOCH};

use blake3::Hasher;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use mugraph_core::{
    error::Error,
    types::{
        CrossNodeMessageRecord, IdempotencyRecord, Response, TransferAuditEvent, XNodeEnvelope,
        XNodeMessageType, validate_envelope_basics,
    },
};
use redb::ReadableTable;
use serde::Serialize;

use crate::{
    database::{CROSS_NODE_MESSAGES, IDEMPOTENCY_KEYS, TRANSFER_AUDIT_LOG},
    lifecycle::{LifecycleEvent, TransferLifecycle},
    peer_registry::PeerRegistry,
    routes::Context,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdempotencyDecision {
    New,
    DuplicateSameRequest,
}

pub fn handle_create(
    request: &XNodeEnvelope<mugraph_core::types::TransferInitPayload>,
    ctx: &Context,
) -> Result<Response, Error> {
    if let Err(e) = enforce_command_security(request, XNodeMessageType::TransferInit, ctx) {
        let _ = audit_reject(ctx, &request.transfer_id, "create_rejected", e.to_string());
        return Err(e);
    }

    Ok(Response::CrossNodeTransferCreate {
        transfer_id: request.transfer_id.clone(),
        accepted: true,
    })
}

pub fn handle_notify(
    request: &XNodeEnvelope<mugraph_core::types::TransferNoticePayload>,
    ctx: &Context,
) -> Result<Response, Error> {
    if let Err(e) = enforce_command_security(request, XNodeMessageType::TransferNotice, ctx) {
        let _ = audit_reject(ctx, &request.transfer_id, "notify_rejected", e.to_string());
        return Err(e);
    }

    Ok(Response::CrossNodeTransferNotify { accepted: true })
}

pub fn handle_status(
    request: &XNodeEnvelope<mugraph_core::types::TransferStatusQueryPayload>,
) -> Result<Response, Error> {
    validate_envelope_basics(request, XNodeMessageType::TransferStatusQuery, 3)
        .map_err(Error::from)?;

    let mut lifecycle = TransferLifecycle::new();
    lifecycle.apply(LifecycleEvent::SourceSubmitted);

    Ok(Response::CrossNodeTransferStatus(Box::new(XNodeEnvelope {
        m: "xnode".to_string(),
        version: request.version.clone(),
        message_type: XNodeMessageType::TransferStatus,
        message_id: request.message_id.clone(),
        transfer_id: request.transfer_id.clone(),
        idempotency_key: request.idempotency_key.clone(),
        correlation_id: request.correlation_id.clone(),
        origin_node_id: request.destination_node_id.clone(),
        destination_node_id: request.origin_node_id.clone(),
        sent_at: request.sent_at.clone(),
        expires_at: None,
        payload: lifecycle.to_status_payload(request.sent_at.clone()),
        auth: request.auth.clone(),
    })))
}

pub fn handle_ack(
    request: &XNodeEnvelope<mugraph_core::types::TransferAckPayload>,
    ctx: &Context,
) -> Result<Response, Error> {
    if let Err(e) = enforce_command_security(request, XNodeMessageType::TransferAck, ctx) {
        let _ = audit_reject(ctx, &request.transfer_id, "ack_rejected", e.to_string());
        return Err(e);
    }

    Ok(Response::CrossNodeTransferAck { accepted: true })
}

fn enforce_command_security<T: Serialize + Clone>(
    request: &XNodeEnvelope<T>,
    expected_message_type: XNodeMessageType,
    ctx: &Context,
) -> Result<IdempotencyDecision, Error> {
    validate_envelope_basics(request, expected_message_type.clone(), 3).map_err(Error::from)?;
    validate_freshness(&request.sent_at, request.expires_at.as_deref())?;
    validate_destination_binding(request)?;
    validate_auth_signature(request, ctx)?;
    check_replay_and_idempotency(request, &format!("{:?}", expected_message_type), ctx)
}

fn validate_freshness(sent_at: &str, expires_at: Option<&str>) -> Result<(), Error> {
    if sent_at.is_empty() || !sent_at.ends_with('Z') {
        return Err(Error::InvalidInput {
            reason: "invalid sent_at timestamp format".to_string(),
        });
    }

    let Some(expires_at) = expires_at else {
        return Err(Error::InvalidInput {
            reason: "expires_at is required for command envelopes".to_string(),
        });
    };

    if expires_at.is_empty() || !expires_at.ends_with('Z') {
        return Err(Error::InvalidInput {
            reason: "invalid expires_at timestamp format".to_string(),
        });
    }

    if expires_at <= sent_at {
        return Err(Error::InvalidInput {
            reason: "expired command envelope".to_string(),
        });
    }

    Ok(())
}

fn validate_destination_binding<T>(request: &XNodeEnvelope<T>) -> Result<(), Error> {
    if request.origin_node_id == request.destination_node_id {
        return Err(Error::InvalidInput {
            reason: "origin and destination nodes must differ".to_string(),
        });
    }

    if !request.origin_node_id.starts_with("node://")
        || !request.destination_node_id.starts_with("node://")
    {
        return Err(Error::InvalidInput {
            reason: "origin/destination node ids must use node:// scheme".to_string(),
        });
    }

    Ok(())
}

fn validate_auth_signature<T: Serialize + Clone>(
    request: &XNodeEnvelope<T>,
    ctx: &Context,
) -> Result<(), Error> {
    if request.auth.alg != "Ed25519" {
        return Err(Error::InvalidInput {
            reason: "unsupported auth.alg".to_string(),
        });
    }

    let Some(path) = ctx.config.xnode_peer_registry_file() else {
        return Err(Error::InvalidInput {
            reason: "xnode peer registry is required for cross-node command auth".to_string(),
        });
    };

    let registry = PeerRegistry::load(path)?;
    registry.validate()?;

    let peer = registry
        .peers
        .iter()
        .find(|p| {
            !p.revoked
                && p.node_id == request.origin_node_id
                && p.kid == request.auth.kid
                && p.auth_alg == request.auth.alg
        })
        .ok_or_else(|| Error::InvalidInput {
            reason: "untrusted origin node or key id".to_string(),
        })?;

    let pubkey = muhex::decode(&peer.public_key_hex).map_err(|e| Error::InvalidInput {
        reason: format!("invalid trusted peer public key hex: {e}"),
    })?;
    let verifying_key = VerifyingKey::from_bytes(
        &pubkey
            .as_slice()
            .try_into()
            .map_err(|_| Error::InvalidInput {
                reason: "trusted peer public key must be 32 bytes".to_string(),
            })?,
    )
    .map_err(|e| Error::InvalidInput {
        reason: format!("invalid trusted peer public key: {e}"),
    })?;

    let sig_bytes = muhex::decode(&request.auth.sig).map_err(|e| Error::InvalidInput {
        reason: format!("invalid auth signature hex: {e}"),
    })?;
    let sig = Signature::try_from(sig_bytes.as_slice()).map_err(|e| Error::InvalidInput {
        reason: format!("invalid auth signature bytes: {e}"),
    })?;

    let payload = canonical_auth_payload(request)?;
    verifying_key.verify(&payload, &sig).map_err(|e| Error::InvalidInput {
        reason: format!("invalid auth signature: {e}"),
    })?;

    Ok(())
}

fn check_replay_and_idempotency<T: Serialize + Clone>(
    request: &XNodeEnvelope<T>,
    message_type: &str,
    ctx: &Context,
) -> Result<IdempotencyDecision, Error> {
    let request_hash = request_hash(request)?;
    let now = now_secs();

    let write_tx = ctx.database.write()?;
    let decision = {
        let mut messages = write_tx.open_table(CROSS_NODE_MESSAGES)?;
        let mut idempotency = write_tx.open_table(IDEMPOTENCY_KEYS)?;

        if messages.get(request.message_id.as_str())?.is_some() {
            return Err(Error::InvalidInput {
                reason: "replay detected: duplicate message_id".to_string(),
            });
        }

        let decision = if let Some(existing) = idempotency.get(request.idempotency_key.as_str())? {
            let existing = existing.value();
            if existing.transfer_id == request.transfer_id
                && existing.message_type == message_type
                && existing.request_hash == request_hash
            {
                IdempotencyDecision::DuplicateSameRequest
            } else {
                return Err(Error::InvalidInput {
                    reason: "idempotency conflict".to_string(),
                });
            }
        } else {
            idempotency.insert(
                request.idempotency_key.as_str(),
                &IdempotencyRecord {
                    idempotency_key: request.idempotency_key.clone(),
                    transfer_id: request.transfer_id.clone(),
                    message_type: message_type.to_string(),
                    request_hash: request_hash.clone(),
                    first_seen_at: now,
                    expires_at: now.saturating_add(300),
                },
            )?;
            IdempotencyDecision::New
        };

        messages.insert(
            request.message_id.as_str(),
            &CrossNodeMessageRecord {
                message_id: request.message_id.clone(),
                transfer_id: request.transfer_id.clone(),
                message_type: message_type.to_string(),
                direction: "inbound".to_string(),
                attempt_count: 1,
                created_at: now,
                updated_at: now,
            },
        )?;

        decision
    };

    write_tx.commit()?;
    Ok(decision)
}

fn request_hash<T: Serialize + Clone>(request: &XNodeEnvelope<T>) -> Result<String, Error> {
    let payload = canonical_idempotency_payload(request)?;
    let mut hasher = Hasher::new();
    hasher.update(&payload);
    Ok(muhex::encode(*hasher.finalize().as_bytes()))
}

fn canonical_auth_payload<T: Serialize + Clone>(request: &XNodeEnvelope<T>) -> Result<Vec<u8>, Error> {
    let mut canonical = request.clone();
    canonical.auth.sig.clear();
    Ok(serde_json::to_vec(&canonical)?)
}

fn canonical_idempotency_payload<T: Serialize + Clone>(
    request: &XNodeEnvelope<T>,
) -> Result<Vec<u8>, Error> {
    let mut canonical = request.clone();
    canonical.message_id.clear();
    canonical.sent_at.clear();
    canonical.expires_at = None;
    canonical.auth.sig.clear();
    Ok(serde_json::to_vec(&canonical)?)
}

fn audit_reject(ctx: &Context, transfer_id: &str, event_type: &str, reason: String) -> Result<(), Error> {
    let write_tx = ctx.database.write()?;
    {
        let mut table = write_tx.open_table(TRANSFER_AUDIT_LOG)?;
        let event_id = format!("{}:{}:{}", transfer_id, event_type, now_nanos());
        table.insert(
            event_id.clone().as_str(),
            &TransferAuditEvent {
                event_id,
                transfer_id: transfer_id.to_string(),
                event_type: event_type.to_string(),
                reason,
                created_at: now_secs(),
            },
        )?;
    }
    write_tx.commit()?;
    Ok(())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};
    use proptest::prelude::*;
    use tempfile::TempDir;

    use mugraph_core::types::{
        TransferAckPayload, TransferAckStatus, TransferChainState, TransferCreditState,
        TransferInitPayload, TransferNoticePayload, TransferNoticeStage, TransferQueryType,
        TransferStatusQueryPayload, XNodeAuth,
    };

    use crate::{config::Config, database::Database};

    use super::*;

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
            deposit_confirm_depth: 15,
            deposit_expiration_blocks: 1440,
            min_deposit_value: None,
            max_tx_size: 16384,
            max_withdrawal_fee: 2_000_000,
            fee_tolerance_pct: 5,
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

        // Keep tempdir alive by leaking during test process lifetime.
        std::mem::forget(db_dir);

        let config = test_config_with_registry(registry_path);
        let keypair = config.keypair().unwrap();

        Context {
            keypair,
            database,
            config,
        }
    }

    fn auth() -> XNodeAuth {
        XNodeAuth {
            alg: "Ed25519".to_string(),
            kid: "k1".to_string(),
            sig: String::new(),
        }
    }

    fn sign_envelope<T: Serialize + Clone>(env: &mut XNodeEnvelope<T>, sk: &SigningKey) {
        let mut canonical = env.clone();
        canonical.auth.sig.clear();
        let payload = serde_json::to_vec(&canonical).unwrap();
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
            sent_at: "2026-02-26T18:00:00Z".to_string(),
            expires_at: Some("2026-02-26T18:05:00Z".to_string()),
            payload: TransferInitPayload {
                asset: "lovelace".to_string(),
                amount: "1".to_string(),
                destination_account_ref: "acct".to_string(),
                source_intent_hash: "hash".to_string(),
            },
            auth: auth(),
        }
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
    fn duplicate_message_id_is_rejected_deterministically() {
        let signer = SigningKey::from_bytes(&[7u8; 32]);
        let registry_dir = TempDir::new().unwrap();
        let registry_path = write_registry(&registry_dir, &signer);
        let ctx = test_ctx(&registry_path);

        let mut request = create_request();
        sign_envelope(&mut request, &signer);

        let first = handle_create(&request, &ctx).unwrap();
        assert!(matches!(first, Response::CrossNodeTransferCreate { accepted: true, .. }));

        let second = handle_create(&request, &ctx).unwrap_err();
        match second {
            Error::InvalidInput { reason } => {
                assert!(reason.contains("replay detected"));
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
            Error::InvalidInput { reason } => assert!(reason.contains("idempotency conflict")),
            _ => panic!("expected InvalidInput conflict"),
        }
    }

    #[test]
    fn status_response_contains_chain_and_credit_state() {
        let request = XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferStatusQuery,
            message_id: "mid".to_string(),
            transfer_id: "tr".to_string(),
            idempotency_key: "ik".to_string(),
            correlation_id: "corr".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: "2026-02-26T18:00:00Z".to_string(),
            expires_at: None,
            payload: TransferStatusQueryPayload {
                query_type: TransferQueryType::Current,
            },
            auth: auth(),
        };

        let response = handle_status(&request).unwrap();
        match response {
            Response::CrossNodeTransferStatus(env) => {
                assert_eq!(env.payload.chain_state, TransferChainState::Submitted);
                assert_eq!(env.payload.credit_state, TransferCreditState::None);
            }
            _ => panic!("unexpected response variant"),
        }
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
            sent_at: "2026-02-26T18:00:00Z".to_string(),
            expires_at: Some("2026-02-26T18:05:00Z".to_string()),
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
}
