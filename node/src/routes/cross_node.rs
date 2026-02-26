use std::time::{SystemTime, UNIX_EPOCH};

use blake3::Hasher;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use mugraph_core::{
    error::Error,
    types::{
        CrossNodeMessageRecord, CrossNodeTransferRecord, IdempotencyRecord, Response,
        TransferAuditEvent, TransferChainState, TransferCreditState, TransferSettlementState,
        TransferStatusPayload, XNodeEnvelope, XNodeMessageType, validate_envelope_basics,
    },
};
use redb::ReadableTable;
use serde::Serialize;

use crate::{
    database::{CROSS_NODE_MESSAGES, IDEMPOTENCY_KEYS, TRANSFER_AUDIT_LOG},
    peer_registry::PeerRegistry,
    routes::Context,
};

const MAX_CLOCK_SKEW_SECS: i64 = 300;
const MAX_COMMAND_EXPIRY_HORIZON_SECS: i64 = 900;
const AUTH_DOMAIN_SEP: &[u8] = b"mugraph_xnode_auth_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdempotencyDecision {
    New,
    DuplicateSameRequest,
}

fn protocol_reject(code: &str, detail: impl Into<String>) -> Error {
    Error::InvalidInput {
        reason: format!("{code}: {}", detail.into()),
    }
}

fn error_code(reason: &str) -> &str {
    reason.split(':').next().unwrap_or("INTERNAL_ERROR")
}

fn emit_receive_metrics(message_type: &str, result: &str) {
    metrics::counter!(
        "mugraph_m3_message_send_total",
        "message_type" => message_type.to_string(),
        "result" => result.to_string()
    )
    .increment(1);
}

fn reject_audit_event(code: &str) -> &'static str {
    match code {
        "IDEMPOTENCY_CONFLICT" => "transfer.idempotency_conflict",
        "REPLAY_DETECTED" => "transfer.replay_rejected",
        _ => "transfer.replay_rejected",
    }
}

pub fn handle_create(
    request: &XNodeEnvelope<mugraph_core::types::TransferInitPayload>,
    ctx: &Context,
) -> Result<Response, Error> {
    let decision = match enforce_command_security(request, XNodeMessageType::TransferInit, ctx) {
        Ok(d) => d,
        Err(e) => {
            let mut event = "transfer.replay_rejected";
            if let Error::InvalidInput { reason } = &e {
                let code = error_code(reason);
                event = reject_audit_event(code);
                if code == "REPLAY_DETECTED" {
                    metrics::counter!("mugraph_m3_replay_rejections_total", "message_type" => "transfer_init".to_string()).increment(1);
                }
                if code == "IDEMPOTENCY_CONFLICT" {
                    metrics::counter!("mugraph_m3_idempotency_conflicts_total", "operation" => "transfer_init".to_string()).increment(1);
                }
                tracing::warn!(
                    transfer_id = %request.transfer_id,
                    message_id = %request.message_id,
                    message_type = "transfer_init",
                    idempotency_key = %request.idempotency_key,
                    error_code = code,
                    "transfer.message.receive"
                );
            }
            emit_receive_metrics("transfer_init", "rejected");
            let _ = audit_reject(ctx, &request.transfer_id, event, e.to_string());
            return Err(e);
        }
    };

    if decision == IdempotencyDecision::New {
        metrics::counter!("mugraph_m3_transfers_initiated_total").increment(1);
        let _ = audit_event(ctx, request, "transfer.initiated", "accepted create command".to_string());
    } else {
        metrics::counter!("mugraph_m3_duplicate_messages_total", "message_type" => "transfer_init".to_string()).increment(1);
    }

    emit_receive_metrics("transfer_init", "accepted");
    tracing::info!(
        transfer_id = %request.transfer_id,
        origin_node_id = %request.origin_node_id,
        destination_node_id = %request.destination_node_id,
        protocol_version = %request.version,
        state = "accepted",
        message_id = %request.message_id,
        message_type = "transfer_init",
        idempotency_key = %request.idempotency_key,
        "transfer.message.receive"
    );

    Ok(Response::CrossNodeTransferCreate {
        transfer_id: request.transfer_id.clone(),
        accepted: true,
    })
}

pub fn handle_notify(
    request: &XNodeEnvelope<mugraph_core::types::TransferNoticePayload>,
    ctx: &Context,
) -> Result<Response, Error> {
    let decision = match enforce_command_security(request, XNodeMessageType::TransferNotice, ctx) {
        Ok(d) => d,
        Err(e) => {
            let mut event = "transfer.replay_rejected";
            if let Error::InvalidInput { reason } = &e {
                let code = error_code(reason);
                event = reject_audit_event(code);
                tracing::warn!(
                    transfer_id = %request.transfer_id,
                    message_id = %request.message_id,
                    message_type = "transfer_notice",
                    idempotency_key = %request.idempotency_key,
                    error_code = code,
                    "transfer.message.receive"
                );
            }
            emit_receive_metrics("transfer_notice", "rejected");
            let _ = audit_reject(ctx, &request.transfer_id, event, e.to_string());
            return Err(e);
        }
    };

    if decision == IdempotencyDecision::DuplicateSameRequest {
        metrics::counter!("mugraph_m3_duplicate_messages_total", "message_type" => "transfer_notice".to_string()).increment(1);
    } else {
        let _ = audit_event(ctx, request, "transfer.notice.accepted", "notice accepted".to_string());
    }

    emit_receive_metrics("transfer_notice", "accepted");
    tracing::info!(
        transfer_id = %request.transfer_id,
        origin_node_id = %request.origin_node_id,
        destination_node_id = %request.destination_node_id,
        protocol_version = %request.version,
        state = "accepted",
        message_id = %request.message_id,
        message_type = "transfer_notice",
        idempotency_key = %request.idempotency_key,
        "transfer.message.receive"
    );

    Ok(Response::CrossNodeTransferNotify { accepted: true })
}

pub fn handle_status(
    request: &XNodeEnvelope<mugraph_core::types::TransferStatusQueryPayload>,
    ctx: &Context,
) -> Result<Response, Error> {
    validate_envelope_basics(request, XNodeMessageType::TransferStatusQuery, 3)
        .map_err(Error::from)?;

    let transfer = {
        let read_tx = ctx.database.read()?;
        let table = read_tx.open_table(crate::database::CROSS_NODE_TRANSFERS)?;
        table
            .get(request.transfer_id.as_str())?
            .map(|v| v.value())
            .ok_or_else(|| protocol_reject("TRANSFER_NOT_FOUND", "transfer not found"))?
    };

    let payload = status_payload_from_record(&transfer);
    metrics::histogram!("mugraph_m3_chain_confirmation_depth").record(payload.confirmations_observed as f64);
    if payload.chain_state == TransferChainState::Invalidated {
        metrics::counter!("mugraph_m3_reorg_events_total", "severity" => "deep".to_string()).increment(1);
    }
    if matches!(payload.settlement_state, TransferSettlementState::Confirmed | TransferSettlementState::Invalidated | TransferSettlementState::ManualReview) {
        metrics::counter!("mugraph_m3_transfers_terminal_total", "terminal_state" => format!("{:?}", payload.settlement_state).to_lowercase()).increment(1);
    }

    tracing::info!(
        transfer_id = %request.transfer_id,
        origin_node_id = %request.origin_node_id,
        destination_node_id = %request.destination_node_id,
        protocol_version = %request.version,
        state = %payload.source_state,
        message_id = %request.message_id,
        message_type = "transfer_status_query",
        tx_hash = ?payload.tx_hash,
        confirmations_observed = payload.confirmations_observed,
        "transfer.chain.confirmation_poll"
    );

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
        payload,
        auth: request.auth.clone(),
    })))
}

pub fn handle_ack(
    request: &XNodeEnvelope<mugraph_core::types::TransferAckPayload>,
    ctx: &Context,
) -> Result<Response, Error> {
    let decision = match enforce_command_security(request, XNodeMessageType::TransferAck, ctx) {
        Ok(d) => d,
        Err(e) => {
            let mut event = "transfer.replay_rejected";
            if let Error::InvalidInput { reason } = &e {
                let code = error_code(reason);
                event = reject_audit_event(code);
                tracing::warn!(
                    transfer_id = %request.transfer_id,
                    message_id = %request.message_id,
                    message_type = "transfer_ack",
                    idempotency_key = %request.idempotency_key,
                    error_code = code,
                    "transfer.message.receive"
                );
            }
            emit_receive_metrics("transfer_ack", "rejected");
            let _ = audit_reject(ctx, &request.transfer_id, event, e.to_string());
            return Err(e);
        }
    };

    if decision == IdempotencyDecision::DuplicateSameRequest {
        metrics::counter!("mugraph_m3_duplicate_messages_total", "message_type" => "transfer_ack".to_string()).increment(1);
    }

    emit_receive_metrics("transfer_ack", "accepted");

    Ok(Response::CrossNodeTransferAck { accepted: true })
}

fn enforce_command_security<T: Serialize + Clone>(
    request: &XNodeEnvelope<T>,
    expected_message_type: XNodeMessageType,
    ctx: &Context,
) -> Result<IdempotencyDecision, Error> {
    validate_envelope_basics(request, expected_message_type.clone(), 3).map_err(Error::from)?;
    validate_freshness(&request.sent_at, request.expires_at.as_deref(), now_secs() as i64)?;
    validate_destination_binding(request, &ctx.config.xnode_node_id())?;
    validate_auth_signature(request, ctx)?;
    check_replay_and_idempotency(request, message_type_key(&expected_message_type), ctx)
}

fn message_type_key(message_type: &XNodeMessageType) -> &'static str {
    match message_type {
        XNodeMessageType::TransferInit => "transfer_init",
        XNodeMessageType::TransferNotice => "transfer_notice",
        XNodeMessageType::TransferStatusQuery => "transfer_status_query",
        XNodeMessageType::TransferStatus => "transfer_status",
        XNodeMessageType::TransferAck => "transfer_ack",
    }
}

/// Deterministic internal->external status mapping for M3 status contract.
fn status_payload_from_record(record: &CrossNodeTransferRecord) -> TransferStatusPayload {
    let chain_state = parse_chain_state(&record.chain_state);
    let credit_state = parse_credit_state(&record.credit_state);

    let source_state = match chain_state {
        TransferChainState::Unknown => "requested",
        TransferChainState::Submitted => "submitted",
        TransferChainState::Confirming => "confirming",
        TransferChainState::Confirmed => "confirmed",
        TransferChainState::Invalidated => "invalidated",
    }
    .to_string();

    let destination_state = match (chain_state.clone(), credit_state.clone()) {
        (_, TransferCreditState::Credited) => "credited",
        (_, TransferCreditState::Eligible) => "credit_eligible",
        (TransferChainState::Invalidated, _) => "invalidated",
        (TransferChainState::Submitted | TransferChainState::Confirming | TransferChainState::Confirmed, _) => {
            "chain_observed"
        }
        (TransferChainState::Unknown, _) => "notice_received",
    }
    .to_string();

    let settlement_state = match chain_state {
        TransferChainState::Unknown => TransferSettlementState::NotSubmitted,
        TransferChainState::Submitted => TransferSettlementState::Submitted,
        TransferChainState::Confirming => TransferSettlementState::Confirming,
        TransferChainState::Confirmed => match credit_state {
            TransferCreditState::Held => TransferSettlementState::ManualReview,
            _ => TransferSettlementState::Confirmed,
        },
        TransferChainState::Invalidated => {
            if credit_state == TransferCreditState::Held {
                TransferSettlementState::ManualReview
            } else {
                TransferSettlementState::Invalidated
            }
        }
    };

    TransferStatusPayload {
        source_state,
        destination_state,
        settlement_state,
        chain_state,
        credit_state,
        tx_hash: record.tx_hash.clone(),
        confirmations_observed: record.confirmations_observed,
        updated_at: record.updated_at.to_string(),
    }
}

fn parse_chain_state(value: &str) -> TransferChainState {
    match value {
        "submitted" => TransferChainState::Submitted,
        "confirming" => TransferChainState::Confirming,
        "confirmed" => TransferChainState::Confirmed,
        "invalidated" => TransferChainState::Invalidated,
        _ => TransferChainState::Unknown,
    }
}

fn parse_credit_state(value: &str) -> TransferCreditState {
    match value {
        "eligible" => TransferCreditState::Eligible,
        "credited" => TransferCreditState::Credited,
        "held" => TransferCreditState::Held,
        "reversed" => TransferCreditState::Reversed,
        _ => TransferCreditState::None,
    }
}

fn validate_freshness(sent_at: &str, expires_at: Option<&str>, now: i64) -> Result<(), Error> {
    let sent_at = DateTime::parse_from_rfc3339(sent_at)
        .map_err(|e| protocol_reject("SCHEMA_VALIDATION_FAILED", format!("invalid sent_at timestamp format: {e}")))?
        .with_timezone(&Utc)
        .timestamp();

    let expires_at = expires_at
        .ok_or_else(|| protocol_reject("SCHEMA_VALIDATION_FAILED", "expires_at is required for command envelopes"))?;
    let expires_at = DateTime::parse_from_rfc3339(expires_at)
        .map_err(|e| protocol_reject("SCHEMA_VALIDATION_FAILED", format!("invalid expires_at timestamp format: {e}")))?
        .with_timezone(&Utc)
        .timestamp();

    if expires_at <= sent_at {
        return Err(protocol_reject("REPLAY_DETECTED", "expired command envelope"));
    }

    if (now - sent_at).abs() > MAX_CLOCK_SKEW_SECS {
        return Err(protocol_reject("REPLAY_DETECTED", "sent_at outside allowed clock skew"));
    }

    if expires_at < now {
        return Err(protocol_reject("REPLAY_DETECTED", "command envelope already expired"));
    }

    if expires_at - sent_at > MAX_COMMAND_EXPIRY_HORIZON_SECS {
        return Err(protocol_reject(
            "SCHEMA_VALIDATION_FAILED",
            "command expiry horizon exceeds policy",
        ));
    }

    Ok(())
}

fn validate_destination_binding<T>(
    request: &XNodeEnvelope<T>,
    local_node_id: &str,
) -> Result<(), Error> {
    if request.origin_node_id == request.destination_node_id {
        return Err(protocol_reject(
            "AUTHZ_DENIED",
            "origin and destination nodes must differ",
        ));
    }

    if !request.origin_node_id.starts_with("node://")
        || !request.destination_node_id.starts_with("node://")
    {
        return Err(protocol_reject(
            "SCHEMA_VALIDATION_FAILED",
            "origin/destination node ids must use node:// scheme",
        ));
    }

    if request.destination_node_id != local_node_id {
        return Err(protocol_reject(
            "AUTHZ_DENIED",
            "destination_node_id does not match local node id",
        ));
    }

    Ok(())
}

fn validate_auth_signature<T: Serialize + Clone>(
    request: &XNodeEnvelope<T>,
    ctx: &Context,
) -> Result<(), Error> {
    if request.auth.alg != "Ed25519" {
        return Err(protocol_reject("AUTHZ_DENIED", "unsupported auth.alg"));
    }

    let Some(path) = ctx.config.xnode_peer_registry_file() else {
        return Err(protocol_reject(
            "AUTHZ_DENIED",
            "xnode peer registry is required for cross-node command auth",
        ));
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
        .ok_or_else(|| protocol_reject("UNKNOWN_KEY_ID", "untrusted origin node or key id"))?;

    let pubkey = muhex::decode(&peer.public_key_hex)
        .map_err(|e| protocol_reject("SCHEMA_VALIDATION_FAILED", format!("invalid trusted peer public key hex: {e}")))?;
    let verifying_key = VerifyingKey::from_bytes(
        &pubkey
            .as_slice()
            .try_into()
            .map_err(|_| protocol_reject("SCHEMA_VALIDATION_FAILED", "trusted peer public key must be 32 bytes"))?,
    )
    .map_err(|e| protocol_reject("SCHEMA_VALIDATION_FAILED", format!("invalid trusted peer public key: {e}")))?;

    let sig_bytes = muhex::decode(&request.auth.sig)
        .map_err(|e| protocol_reject("INVALID_SIGNATURE", format!("invalid auth signature hex: {e}")))?;
    let sig = Signature::try_from(sig_bytes.as_slice())
        .map_err(|e| protocol_reject("INVALID_SIGNATURE", format!("invalid auth signature bytes: {e}")))?;

    let payload = canonical_auth_payload(request)?;
    verifying_key
        .verify(&payload, &sig)
        .map_err(|e| protocol_reject("INVALID_SIGNATURE", format!("invalid auth signature: {e}")))?;

    Ok(())
}

fn check_replay_and_idempotency<T: Serialize + Clone>(
    request: &XNodeEnvelope<T>,
    message_type: &str,
    ctx: &Context,
) -> Result<IdempotencyDecision, Error> {
    let request_hash = request_hash(request)?;
    let now = now_secs();
    let tuple_key = format!(
        "{}::{}::{}::{}",
        request.origin_node_id, request.transfer_id, message_type, request.idempotency_key
    );

    let write_tx = ctx.database.write()?;
    let decision = {
        let mut messages = write_tx.open_table(CROSS_NODE_MESSAGES)?;
        let mut idempotency = write_tx.open_table(IDEMPOTENCY_KEYS)?;

        if messages.get(request.message_id.as_str())?.is_some() {
            metrics::counter!(
                "mugraph_m3_replay_rejections_total",
                "message_type" => message_type.to_string()
            )
            .increment(1);
            return Err(protocol_reject(
                "REPLAY_DETECTED",
                "duplicate message_id",
            ));
        }

        let decision = if let Some(existing) = idempotency.get(tuple_key.as_str())? {
            let existing = existing.value();
            if existing.request_hash == request_hash {
                IdempotencyDecision::DuplicateSameRequest
            } else {
                metrics::counter!(
                    "mugraph_m3_idempotency_conflicts_total",
                    "operation" => message_type.to_string()
                )
                .increment(1);
                return Err(protocol_reject("IDEMPOTENCY_CONFLICT", "idempotency conflict"));
            }
        } else {
            idempotency.insert(
                tuple_key.as_str(),
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

    let body = serde_json::to_vec(&canonical)?;
    let mut payload = Vec::with_capacity(AUTH_DOMAIN_SEP.len() + body.len());
    payload.extend_from_slice(AUTH_DOMAIN_SEP);
    payload.extend_from_slice(&body);
    Ok(payload)
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

fn audit_event<T>(
    ctx: &Context,
    request: &XNodeEnvelope<T>,
    event_type: &str,
    reason: String,
) -> Result<(), Error> {
    let write_tx = ctx.database.write()?;
    {
        let mut table = write_tx.open_table(TRANSFER_AUDIT_LOG)?;
        let event_id = format!("{}:{}:{}", request.transfer_id, event_type, now_nanos());
        table.insert(
            event_id.clone().as_str(),
            &TransferAuditEvent {
                event_id,
                transfer_id: request.transfer_id.clone(),
                event_type: event_type.to_string(),
                reason: format!(
                    "{} | origin={} destination={} protocol={} message_id={} message_type={} idempotency_key={}",
                    reason,
                    request.origin_node_id,
                    request.destination_node_id,
                    request.version,
                    request.message_id,
                    message_type_key(&request.message_type),
                    request.idempotency_key
                ),
                created_at: now_secs(),
            },
        )?;
    }
    write_tx.commit()?;
    Ok(())
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
    use chrono::Utc;
    use ed25519_dalek::{Signer, SigningKey};
    use proptest::prelude::*;
    use tempfile::TempDir;

    use mugraph_core::types::{
        CrossNodeTransferRecord, TransferAckPayload, TransferAckStatus, TransferChainState,
        TransferCreditState, TransferInitPayload, TransferNoticePayload, TransferNoticeStage,
        TransferSettlementState,
        TransferQueryType, TransferStatusQueryPayload, XNodeAuth,
    };

    use crate::{config::Config, database::{CROSS_NODE_TRANSFERS, Database}};

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
            xnode_node_id: "node://b".to_string(),
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

    fn now_rfc3339_offset(secs: i64) -> String {
        (Utc::now() + chrono::Duration::seconds(secs)).to_rfc3339()
    }

    fn sign_envelope<T: Serialize + Clone>(env: &mut XNodeEnvelope<T>, sk: &SigningKey) {
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

    fn seed_transfer(ctx: &Context, transfer_id: &str, chain_state: &str, credit_state: &str) {
        let w = ctx.database.write().unwrap();
        {
            let mut table = w.open_table(CROSS_NODE_TRANSFERS).unwrap();
            table.insert(
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
            Error::InvalidInput { reason } => assert!(reason.contains("IDEMPOTENCY_CONFLICT")),
            _ => panic!("expected InvalidInput conflict"),
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
            Error::InvalidInput { reason } => assert!(reason.contains("AUTHZ_DENIED")),
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
            Error::InvalidInput { reason } => assert!(reason.contains("REPLAY_DETECTED")),
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
        request.expires_at = Some(now_rfc3339_offset(MAX_COMMAND_EXPIRY_HORIZON_SECS + 60));
        sign_envelope(&mut request, &signer);

        let err = handle_create(&request, &ctx).unwrap_err();
        match err {
            Error::InvalidInput { reason } => assert!(reason.contains("SCHEMA_VALIDATION_FAILED")),
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
            sent_at: now_rfc3339_offset(0),
            expires_at: None,
            payload: TransferStatusQueryPayload {
                query_type: TransferQueryType::Current,
            },
            auth: auth(),
        };

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
        assert_eq!(payload.settlement_state, TransferSettlementState::ManualReview);
        assert_eq!(payload.chain_state, TransferChainState::Invalidated);
        assert_eq!(payload.credit_state, TransferCreditState::Held);
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
}
