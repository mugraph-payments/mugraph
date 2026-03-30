use blake3::Hasher;
use mugraph_core::{
    error::Error,
    types::{CrossNodeMessageRecord, IdempotencyRecord, XNodeEnvelope},
};
use redb::ReadableTable;
use serde::Serialize;

use super::{now_secs, protocol_reject};
use crate::{
    database::{CROSS_NODE_MESSAGES, IDEMPOTENCY_KEYS},
    routes::Context,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IdempotencyDecision {
    New,
    DuplicateSameRequest,
}

pub(super) fn check_replay_and_idempotency<T: Serialize + Clone>(
    request: &XNodeEnvelope<T>,
    message_type: &str,
    ctx: &Context,
) -> Result<IdempotencyDecision, Error> {
    let request_hash = request_hash(request)?;
    let now = now_secs();
    let tuple_key = format!(
        "{}::{}::{}::{}",
        request.origin_node_id,
        request.transfer_id,
        message_type,
        request.idempotency_key
    );

    let write_tx = ctx.database.write()?;
    let decision = {
        let mut messages = write_tx.open_table(CROSS_NODE_MESSAGES)?;
        let mut idempotency = write_tx.open_table(IDEMPOTENCY_KEYS)?;

        if messages.get(request.message_id.as_str())?.is_some() {
            metrics::counter!(
                "mugraph_replay_rejections_total",
                "message_type" => message_type.to_string()
            )
            .increment(1);
            return Err(protocol_reject(
                "REPLAY_DETECTED",
                "duplicate message_id",
            ));
        }

        let existing = idempotency.get(tuple_key.as_str())?.map(|v| v.value());
        let decision = if let Some(existing) = existing {
            if existing.expires_at <= now {
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
            } else if existing.request_hash == request_hash {
                IdempotencyDecision::DuplicateSameRequest
            } else {
                metrics::counter!(
                    "mugraph_idempotency_conflicts_total",
                    "operation" => message_type.to_string()
                )
                .increment(1);
                return Err(protocol_reject(
                    "IDEMPOTENCY_CONFLICT",
                    "idempotency conflict",
                ));
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

fn request_hash<T: Serialize + Clone>(
    request: &XNodeEnvelope<T>,
) -> Result<String, Error> {
    let payload = canonical_idempotency_payload(request)?;
    let mut hasher = Hasher::new();
    hasher.update(&payload);
    Ok(muhex::encode(*hasher.finalize().as_bytes()))
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
