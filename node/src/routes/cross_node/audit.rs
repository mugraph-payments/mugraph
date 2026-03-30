use mugraph_core::{
    error::Error,
    types::{
        CrossNodeTransferRecord,
        TransferAuditEvent,
        TransferChainState,
        TransferCreditState,
        TransferSettlementState,
        TransferStatusPayload,
        XNodeEnvelope,
    },
};

use super::{message_type_key, now_nanos, now_secs};
use crate::{database::TRANSFER_AUDIT_LOG, routes::Context};

pub(super) fn emit_chain_metrics(
    record: &CrossNodeTransferRecord,
    payload: &TransferStatusPayload,
) {
    metrics::histogram!("mugraph_chain_confirmation_depth")
        .record(payload.confirmations_observed as f64);
    metrics::histogram!("mugraph_settlement_latency_seconds")
        .record(record.updated_at.saturating_sub(record.created_at) as f64);

    let result = if record.tx_hash.is_some() {
        "observed"
    } else {
        "missing"
    };
    metrics::counter!(
        "mugraph_chain_submission_total",
        "result" => result.to_string(),
        "provider" => "local_store".to_string()
    )
    .increment(1);

    if payload.chain_state == TransferChainState::Invalidated {
        metrics::counter!("mugraph_reorg_events_total", "severity" => "deep".to_string())
            .increment(1);
    }
    if matches!(
        payload.settlement_state,
        TransferSettlementState::Confirmed
            | TransferSettlementState::Invalidated
            | TransferSettlementState::ManualReview
    ) {
        metrics::counter!("mugraph_transfers_terminal_total", "terminal_state" => format!("{:?}", payload.settlement_state).to_lowercase()).increment(1);
    }
}

pub(super) fn audit_status_events_with<T, F>(
    _ctx: &Context,
    _request: &XNodeEnvelope<T>,
    payload: &TransferStatusPayload,
    mut emit: F,
) -> Result<(), Error>
where
    F: FnMut(&str, String) -> Result<(), Error>,
{
    if payload.credit_state == TransferCreditState::Credited {
        emit("transfer.credited", "credit observed in status".to_string())?;
    }
    if payload.settlement_state == TransferSettlementState::Confirmed {
        emit("transfer.confirmed", "confirmed in status".to_string())?;
    }
    if payload.chain_state == TransferChainState::Invalidated {
        emit("transfer.invalidated", "invalidated in status".to_string())?;
    }
    if payload.settlement_state == TransferSettlementState::ManualReview {
        emit(
            "transfer.manual_override",
            "manual-review state observed".to_string(),
        )?;
    }

    Ok(())
}

pub(super) fn audit_status_events<T>(
    ctx: &Context,
    request: &XNodeEnvelope<T>,
    payload: &TransferStatusPayload,
) -> Result<(), Error> {
    audit_status_events_with(ctx, request, payload, |event_type, reason| {
        audit_event(ctx, request, event_type, reason)
    })
}

pub(super) fn audit_event<T>(
    ctx: &Context,
    request: &XNodeEnvelope<T>,
    event_type: &str,
    reason: String,
) -> Result<(), Error> {
    let write_tx = ctx.database.write()?;
    {
        let mut table = write_tx.open_table(TRANSFER_AUDIT_LOG)?;
        let event_id =
            format!("{}:{}:{}", request.transfer_id, event_type, now_nanos());
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

pub(super) fn audit_reject(
    ctx: &Context,
    transfer_id: &str,
    event_type: &str,
    reason: String,
) -> Result<(), Error> {
    let write_tx = ctx.database.write()?;
    {
        let mut table = write_tx.open_table(TRANSFER_AUDIT_LOG)?;
        let event_id =
            format!("{}:{}:{}", transfer_id, event_type, now_nanos());
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
