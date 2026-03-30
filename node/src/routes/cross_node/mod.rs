use std::time::{SystemTime, UNIX_EPOCH};

use mugraph_core::{
    error::Error,
    types::{
        CrossNodeTransferRecord,
        Response,
        XNodeEnvelope,
        XNodeMessageType,
        validate_envelope_basics,
    },
};
use redb::ReadableTable;
use serde::Serialize;

use crate::{
    database::CROSS_NODE_TRANSFERS,
    lifecycle::status_payload_from_record,
    routes::Context,
};

mod audit;
mod auth;
mod idempotency;
mod status;

#[cfg(test)]
mod tests;

use self::{
    audit::{
        audit_event,
        audit_reject,
        audit_status_events,
        emit_chain_metrics,
    },
    auth::{
        validate_auth_signature,
        validate_destination_binding,
        validate_freshness,
        validate_query_freshness,
    },
    idempotency::{IdempotencyDecision, check_replay_and_idempotency},
    status::sign_status_response,
};

fn protocol_reject(code: &str, detail: impl Into<String>) -> Error {
    Error::InvalidInput {
        reason: format!("{code}: {}", detail.into()),
    }
}

fn error_code(reason: &str) -> &str {
    reason.split(':').next().unwrap_or("INTERNAL_ERROR")
}

const M3_MESSAGE_RECEIVE_COUNTER: &str = "mugraph_message_receive_total";

fn emit_receive_metrics(message_type: &str, result: &str) {
    metrics::counter!(
        M3_MESSAGE_RECEIVE_COUNTER,
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
    let decision = match enforce_command_security(
        request,
        XNodeMessageType::TransferInit,
        ctx,
    ) {
        Ok(d) => d,
        Err(e) => {
            let mut event = "transfer.replay_rejected";
            if let Error::InvalidInput { reason } = &e {
                let code = error_code(reason);
                event = reject_audit_event(code);
                if code == "REPLAY_DETECTED" {
                    metrics::counter!("mugraph_replay_rejections_total", "message_type" => "transfer_init".to_string()).increment(1);
                }
                if code == "IDEMPOTENCY_CONFLICT" {
                    metrics::counter!("mugraph_idempotency_conflicts_total", "operation" => "transfer_init".to_string()).increment(1);
                }
                tracing::warn!(
                    transfer_id = %request.transfer_id,
                    origin_node_id = %request.origin_node_id,
                    destination_node_id = %request.destination_node_id,
                    protocol_version = %request.version,
                    state = "rejected",
                    message_id = %request.message_id,
                    message_type = "transfer_init",
                    idempotency_key = %request.idempotency_key,
                    error_code = code,
                    "transfer.message.receive"
                );
            }
            emit_receive_metrics("transfer_init", "rejected");
            if let Err(audit_err) =
                audit_reject(ctx, &request.transfer_id, event, e.to_string())
            {
                tracing::error!(
                    "failed to write reject audit event: {}",
                    audit_err
                );
            }
            return Err(e);
        }
    };

    if decision == IdempotencyDecision::New {
        let now = now_secs();
        let write_tx = ctx.database.write()?;
        {
            let mut transfers = write_tx.open_table(CROSS_NODE_TRANSFERS)?;
            if transfers.get(request.transfer_id.as_str())?.is_some() {
                return Err(protocol_reject(
                    "TRANSFER_ALREADY_EXISTS",
                    "transfer_id already exists",
                ));
            }

            transfers.insert(
                request.transfer_id.as_str(),
                &CrossNodeTransferRecord {
                    transfer_id: request.transfer_id.clone(),
                    source_node_id: request.origin_node_id.clone(),
                    destination_node_id: request.destination_node_id.clone(),
                    tx_hash: None,
                    chain_state: "unknown".to_string(),
                    credit_state: "none".to_string(),
                    confirmations_observed: 0,
                    created_at: now,
                    updated_at: now,
                },
            )?;
        }
        write_tx.commit()?;

        metrics::counter!("mugraph_transfers_initiated_total").increment(1);
        audit_event(
            ctx,
            request,
            "transfer.initiated",
            "accepted create command".to_string(),
        )?;
    } else {
        metrics::counter!("mugraph_duplicate_messages_total", "message_type" => "transfer_init".to_string()).increment(1);
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
    let decision = match enforce_command_security(
        request,
        XNodeMessageType::TransferNotice,
        ctx,
    ) {
        Ok(d) => d,
        Err(e) => {
            let mut event = "transfer.replay_rejected";
            if let Error::InvalidInput { reason } = &e {
                let code = error_code(reason);
                event = reject_audit_event(code);
                tracing::warn!(
                    transfer_id = %request.transfer_id,
                    origin_node_id = %request.origin_node_id,
                    destination_node_id = %request.destination_node_id,
                    protocol_version = %request.version,
                    state = "rejected",
                    message_id = %request.message_id,
                    message_type = "transfer_notice",
                    idempotency_key = %request.idempotency_key,
                    error_code = code,
                    "transfer.message.receive"
                );
            }
            emit_receive_metrics("transfer_notice", "rejected");
            if let Err(audit_err) =
                audit_reject(ctx, &request.transfer_id, event, e.to_string())
            {
                tracing::error!(
                    "failed to write reject audit event: {}",
                    audit_err
                );
            }
            return Err(e);
        }
    };

    {
        let read_tx = ctx.database.read()?;
        let transfers = read_tx.open_table(CROSS_NODE_TRANSFERS)?;
        if transfers.get(request.transfer_id.as_str())?.is_none() {
            return Err(protocol_reject(
                "TRANSFER_NOT_FOUND",
                "cannot accept notice for unknown transfer",
            ));
        }
    }

    if decision == IdempotencyDecision::DuplicateSameRequest {
        metrics::counter!("mugraph_duplicate_messages_total", "message_type" => "transfer_notice".to_string()).increment(1);
    } else {
        let now = now_secs();
        let write_tx = ctx.database.write()?;
        {
            let mut transfers = write_tx.open_table(CROSS_NODE_TRANSFERS)?;
            let existing = {
                transfers
                    .get(request.transfer_id.as_str())?
                    .map(|v| v.value())
            };

            if let Some(mut updated) = existing {
                updated.tx_hash = Some(request.payload.tx_hash.clone());
                updated.confirmations_observed = request
                    .payload
                    .confirmations
                    .unwrap_or(updated.confirmations_observed)
                    .max(updated.confirmations_observed);
                updated.chain_state = match request.payload.notice_stage {
                    mugraph_core::types::TransferNoticeStage::Submitted => {
                        "submitted"
                    }
                    mugraph_core::types::TransferNoticeStage::Confirmed => {
                        "confirming"
                    }
                    mugraph_core::types::TransferNoticeStage::Finalized => {
                        "confirmed"
                    }
                }
                .to_string();
                updated.updated_at = now;
                transfers.insert(request.transfer_id.as_str(), &updated)?;
            }
        }
        write_tx.commit()?;

        audit_event(
            ctx,
            request,
            "transfer.notice.accepted",
            "notice accepted".to_string(),
        )?;
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
    enforce_query_security(
        request,
        XNodeMessageType::TransferStatusQuery,
        ctx,
    )?;

    let transfer = {
        let read_tx = ctx.database.read()?;
        let table =
            read_tx.open_table(crate::database::CROSS_NODE_TRANSFERS)?;
        table
            .get(request.transfer_id.as_str())?
            .map(|v| v.value())
            .ok_or_else(|| {
                protocol_reject("TRANSFER_NOT_FOUND", "transfer not found")
            })?
    };

    let payload = status_payload_from_record(&transfer);
    emit_chain_metrics(&transfer, &payload);
    audit_status_events(ctx, request, &payload)?;

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

    let mut response_envelope = XNodeEnvelope {
        m: "xnode".to_string(),
        version: request.version.clone(),
        message_type: XNodeMessageType::TransferStatus,
        message_id: request.message_id.clone(),
        transfer_id: request.transfer_id.clone(),
        idempotency_key: request.idempotency_key.clone(),
        correlation_id: request.correlation_id.clone(),
        origin_node_id: request.destination_node_id.clone(),
        destination_node_id: request.origin_node_id.clone(),
        sent_at: chrono::Utc::now().to_rfc3339(),
        expires_at: None,
        payload,
        auth: mugraph_core::types::XNodeAuth {
            alg: "Ed25519".to_string(),
            kid: ctx.config.xnode_node_id(),
            sig: String::new(),
        },
    };

    sign_status_response(&mut response_envelope, ctx)?;

    Ok(Response::CrossNodeTransferStatus(Box::new(
        response_envelope,
    )))
}

pub fn handle_ack(
    request: &XNodeEnvelope<mugraph_core::types::TransferAckPayload>,
    ctx: &Context,
) -> Result<Response, Error> {
    let decision = match enforce_command_security(
        request,
        XNodeMessageType::TransferAck,
        ctx,
    ) {
        Ok(d) => d,
        Err(e) => {
            let mut event = "transfer.replay_rejected";
            if let Error::InvalidInput { reason } = &e {
                let code = error_code(reason);
                event = reject_audit_event(code);
                tracing::warn!(
                    transfer_id = %request.transfer_id,
                    origin_node_id = %request.origin_node_id,
                    destination_node_id = %request.destination_node_id,
                    protocol_version = %request.version,
                    state = "rejected",
                    message_id = %request.message_id,
                    message_type = "transfer_ack",
                    idempotency_key = %request.idempotency_key,
                    error_code = code,
                    "transfer.message.receive"
                );
            }
            emit_receive_metrics("transfer_ack", "rejected");
            if let Err(audit_err) =
                audit_reject(ctx, &request.transfer_id, event, e.to_string())
            {
                tracing::error!(
                    "failed to write reject audit event: {}",
                    audit_err
                );
            }
            return Err(e);
        }
    };

    if decision == IdempotencyDecision::DuplicateSameRequest {
        metrics::counter!("mugraph_duplicate_messages_total", "message_type" => "transfer_ack".to_string()).increment(1);
    }

    emit_receive_metrics("transfer_ack", "accepted");

    Ok(Response::CrossNodeTransferAck { accepted: true })
}

fn enforce_command_security<T: Serialize + Clone>(
    request: &XNodeEnvelope<T>,
    expected_message_type: XNodeMessageType,
    ctx: &Context,
) -> Result<IdempotencyDecision, Error> {
    validate_envelope_basics(request, expected_message_type.clone(), 3)
        .map_err(Error::from)?;
    validate_freshness(
        &request.sent_at,
        request.expires_at.as_deref(),
        now_secs() as i64,
    )?;
    validate_destination_binding(request, &ctx.config.xnode_node_id())?;
    validate_auth_signature(request, ctx)?;
    check_replay_and_idempotency(
        request,
        message_type_key(&expected_message_type),
        ctx,
    )
}

fn enforce_query_security<T: Serialize + Clone>(
    request: &XNodeEnvelope<T>,
    expected_message_type: XNodeMessageType,
    ctx: &Context,
) -> Result<(), Error> {
    validate_envelope_basics(request, expected_message_type, 3)
        .map_err(Error::from)?;
    validate_query_freshness(&request.sent_at, now_secs() as i64)?;
    validate_destination_binding(request, &ctx.config.xnode_node_id())?;
    validate_auth_signature(request, ctx)?;
    Ok(())
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
