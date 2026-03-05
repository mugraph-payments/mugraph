use std::{sync::Arc, time::Duration};

use blake3::Hasher;
use mugraph_core::{
    error::Error,
    types::{CrossNodeMessageRecord, TransferAuditEvent},
};
use redb::ReadableTable;
use tokio::time::{MissedTickBehavior, interval};

use crate::database::{CROSS_NODE_MESSAGES, CROSS_NODE_TRANSFERS, Database, TRANSFER_AUDIT_LOG};

const DEFAULT_MAX_ATTEMPTS: u32 = 12;
const BASE_BACKOFF_SECS: u64 = 2;
const MAX_BACKOFF_SECS: u64 = 300;

#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_backoff_secs: u64,
    pub max_backoff_secs: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            base_backoff_secs: BASE_BACKOFF_SECS,
            max_backoff_secs: MAX_BACKOFF_SECS,
        }
    }
}

pub async fn reconciler_loop(database: Arc<Database>, tick: Duration, policy: RetryPolicy) {
    let mut ticker = interval(tick);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        ticker.tick().await;
        if let Err(e) = reconcile_once(&database, policy, now_secs()) {
            tracing::error!("reconciler tick failed: {}", e);
        }
    }
}

pub fn reconcile_once(database: &Database, policy: RetryPolicy, now: u64) -> Result<(), Error> {
    let mut pending = Vec::new();

    {
        let read_tx = database.read()?;
        let messages = read_tx.open_table(CROSS_NODE_MESSAGES)?;

        for row in messages.iter()? {
            let (_k, v) = row?;
            let message = v.value();
            if message.direction != "outbound" {
                continue;
            }

            if !matches!(
                message.message_type.as_str(),
                "transfer_notice" | "transfer_status_query" | "transfer_ack"
            ) {
                continue;
            }

            if message.attempt_count >= policy.max_attempts {
                pending.push(RetryAction::Exhausted(message));
                continue;
            }

            let due_at = message.updated_at.saturating_add(next_retry_delay_secs(&message, policy));
            if now >= due_at {
                pending.push(RetryAction::Retry(message));
            }
        }
    }

    if pending.is_empty() {
        return Ok(());
    }

    let write_tx = database.write()?;
    {
        let mut messages = write_tx.open_table(CROSS_NODE_MESSAGES)?;
        let mut transfers = write_tx.open_table(CROSS_NODE_TRANSFERS)?;
        let mut audits = write_tx.open_table(TRANSFER_AUDIT_LOG)?;

        for action in pending {
            match action {
                RetryAction::Retry(mut message) => {
                    message.attempt_count = message.attempt_count.saturating_add(1);
                    message.updated_at = now;

                    let exhausted_after_increment = message.attempt_count >= policy.max_attempts;
                    if exhausted_after_increment {
                        message.direction = "terminal".to_string();
                    }
                    messages.insert(message.message_id.as_str(), &message)?;

                    metrics::counter!(
                        "mugraph_m3_message_retries_total",
                        "message_type" => message.message_type.clone(),
                        "reason" => if exhausted_after_increment { "exhausted".to_string() } else { "scheduled".to_string() }
                    )
                    .increment(1);

                    tracing::info!(
                        transfer_id = %message.transfer_id,
                        message_id = %message.message_id,
                        message_type = %message.message_type,
                        attempt_no = message.attempt_count,
                        state = if exhausted_after_increment { "exhausted" } else { "retry_scheduled" },
                        "transfer.reconcile"
                    );

                    if exhausted_after_increment {
                        handle_exhaustion(&message, now, &mut transfers, &mut audits)?;
                    } else {
                        write_audit(
                            &mut audits,
                            &message.transfer_id,
                            "reconciler.retry_scheduled",
                            format!(
                                "retrying {} attempt {}",
                                message.message_type, message.attempt_count
                            ),
                            now,
                        )?;
                    }
                }
                RetryAction::Exhausted(mut message) => {
                    metrics::counter!(
                        "mugraph_m3_message_retries_total",
                        "message_type" => message.message_type.clone(),
                        "reason" => "already_exhausted".to_string()
                    )
                    .increment(1);
                    handle_exhaustion(&message, now, &mut transfers, &mut audits)?;
                    message.direction = "terminal".to_string();
                    message.updated_at = now;
                    messages.insert(message.message_id.as_str(), &message)?;
                }
            }
        }
    }
    write_tx.commit()?;
    emit_stuck_transfer_gauges(database)?;

    Ok(())
}

fn handle_exhaustion(
    message: &CrossNodeMessageRecord,
    now: u64,
    transfers: &mut redb::Table<'_, &str, mugraph_core::types::CrossNodeTransferRecord>,
    audits: &mut redb::Table<'_, &str, TransferAuditEvent>,
) -> Result<(), Error> {
    // Lost ACK is advisory and must not block convergence.
    if message.message_type == "transfer_ack" {
        tracing::warn!(
            transfer_id = %message.transfer_id,
            message_id = %message.message_id,
            message_type = %message.message_type,
            attempt_no = message.attempt_count,
            error_code = "ACK_EXHAUSTED",
            "transfer.reconcile"
        );
        return write_audit(
            audits,
            &message.transfer_id,
            "reconciler.ack_exhausted",
            "ack retries exhausted; convergence remains chain-authoritative".to_string(),
            now,
        );
    }

    let maybe_transfer = {
        transfers
            .get(message.transfer_id.as_str())?
            .map(|guard| guard.value())
    };

    if let Some(mut transfer) = maybe_transfer {
        transfer.credit_state = "held".to_string();
        if transfer.chain_state != "confirmed" {
            transfer.chain_state = "invalidated".to_string();
        }
        transfer.updated_at = now;
        transfers.insert(message.transfer_id.as_str(), &transfer)?;
    }

    metrics::counter!(
        "mugraph_m3_transfers_terminal_total",
        "terminal_state" => "manual_review".to_string()
    )
    .increment(1);

    tracing::warn!(
        transfer_id = %message.transfer_id,
        message_id = %message.message_id,
        message_type = %message.message_type,
        attempt_no = message.attempt_count,
        error_code = "RETRY_EXHAUSTED",
        state = "manual_review",
        "transfer.reconcile"
    );

    write_audit(
        audits,
        &message.transfer_id,
        "reconciler.manual_review",
        format!(
            "retry exhaustion for {} after {} attempts",
            message.message_type, message.attempt_count
        ),
        now,
    )
}

fn emit_stuck_transfer_gauges(database: &Database) -> Result<(), Error> {
    let read_tx = database.read()?;
    let transfers = read_tx.open_table(CROSS_NODE_TRANSFERS)?;

    let mut held = 0u64;
    let mut invalidated = 0u64;

    for row in transfers.iter()? {
        let (_k, v) = row?;
        let transfer = v.value();
        if transfer.credit_state == "held" {
            held += 1;
        }
        if transfer.chain_state == "invalidated" {
            invalidated += 1;
        }
    }

    metrics::gauge!("mugraph_m3_stuck_transfers_gauge", "state" => "held".to_string()).set(held as f64);
    metrics::gauge!("mugraph_m3_stuck_transfers_gauge", "state" => "invalidated".to_string()).set(invalidated as f64);
    Ok(())
}

fn write_audit(
    audits: &mut redb::Table<'_, &str, TransferAuditEvent>,
    transfer_id: &str,
    event_type: &str,
    reason: String,
    now: u64,
) -> Result<(), Error> {
    let event_id = format!("{}:{}:{}", transfer_id, event_type, now_nanos());
    audits.insert(
        event_id.clone().as_str(),
        &TransferAuditEvent {
            event_id,
            transfer_id: transfer_id.to_string(),
            event_type: event_type.to_string(),
            reason,
            created_at: now,
        },
    )?;
    Ok(())
}

enum RetryAction {
    Retry(CrossNodeMessageRecord),
    Exhausted(CrossNodeMessageRecord),
}

fn next_retry_delay_secs(message: &CrossNodeMessageRecord, policy: RetryPolicy) -> u64 {
    let exp = message.attempt_count.saturating_sub(1).min(31);
    let backoff = policy
        .base_backoff_secs
        .saturating_mul(1u64 << exp)
        .min(policy.max_backoff_secs);

    let jitter = deterministic_jitter_secs(message, backoff);
    backoff.saturating_add(jitter)
}

fn deterministic_jitter_secs(message: &CrossNodeMessageRecord, base: u64) -> u64 {
    if base == 0 {
        return 0;
    }

    let mut hasher = Hasher::new();
    hasher.update(message.message_id.as_bytes());
    hasher.update(message.transfer_id.as_bytes());

    let digest = hasher.finalize();
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&digest.as_bytes()[..8]);
    let rnd = u64::from_le_bytes(buf);

    // 0..=25% jitter
    let max_jitter = (base / 4).max(1);
    rnd % (max_jitter + 1)
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn now_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use mugraph_core::types::CrossNodeTransferRecord;
    use proptest::prelude::*;

    use crate::database::CROSS_NODE_TRANSFERS;

    use super::*;

    fn temp_db() -> Database {
        let path = std::env::temp_dir().join(format!(
            "mugraph-reconciler-test-{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let db = Database::setup(path).unwrap();
        db.migrate().unwrap();
        db
    }

    fn seed_transfer(db: &Database, transfer_id: &str) {
        let w = db.write().unwrap();
        {
            let mut t = w.open_table(CROSS_NODE_TRANSFERS).unwrap();
            t.insert(
                transfer_id,
                &CrossNodeTransferRecord {
                    transfer_id: transfer_id.to_string(),
                    source_node_id: "node://a".to_string(),
                    destination_node_id: "node://b".to_string(),
                    tx_hash: Some("abcd".to_string()),
                    chain_state: "confirming".to_string(),
                    credit_state: "none".to_string(),
                    confirmations_observed: 2,
                    created_at: 1,
                    updated_at: 1,
                },
            )
            .unwrap();
        }
        w.commit().unwrap();
    }

    fn seed_message(db: &Database, message: CrossNodeMessageRecord) {
        let w = db.write().unwrap();
        {
            let mut m = w.open_table(CROSS_NODE_MESSAGES).unwrap();
            m.insert(message.message_id.as_str(), &message).unwrap();
        }
        w.commit().unwrap();
    }

    #[test]
    fn exhausted_notice_transitions_transfer_to_manual_review_path() {
        let db = temp_db();
        seed_transfer(&db, "tr-1");
        seed_message(
            &db,
            CrossNodeMessageRecord {
                message_id: "mid-1".to_string(),
                transfer_id: "tr-1".to_string(),
                message_type: "transfer_notice".to_string(),
                direction: "outbound".to_string(),
                attempt_count: 12,
                created_at: 1,
                updated_at: 1,
            },
        );

        reconcile_once(&db, RetryPolicy::default(), 10).unwrap();

        let r = db.read().unwrap();
        let t = r.open_table(CROSS_NODE_TRANSFERS).unwrap();
        let transfer = t.get("tr-1").unwrap().unwrap().value();
        assert_eq!(transfer.credit_state, "held");
        assert_eq!(transfer.chain_state, "invalidated");
    }

    #[test]
    fn exhausted_ack_does_not_mutate_transfer_terminality() {
        let db = temp_db();
        seed_transfer(&db, "tr-2");
        seed_message(
            &db,
            CrossNodeMessageRecord {
                message_id: "mid-2".to_string(),
                transfer_id: "tr-2".to_string(),
                message_type: "transfer_ack".to_string(),
                direction: "outbound".to_string(),
                attempt_count: 12,
                created_at: 1,
                updated_at: 1,
            },
        );

        reconcile_once(&db, RetryPolicy::default(), 10).unwrap();

        let r = db.read().unwrap();
        let t = r.open_table(CROSS_NODE_TRANSFERS).unwrap();
        let transfer = t.get("tr-2").unwrap().unwrap().value();
        assert_eq!(transfer.credit_state, "none");
        assert_eq!(transfer.chain_state, "confirming");
    }

    #[test]
    fn exhausted_message_is_terminalized_and_not_reprocessed() {
        let db = temp_db();
        seed_transfer(&db, "tr-3");
        seed_message(
            &db,
            CrossNodeMessageRecord {
                message_id: "mid-3".to_string(),
                transfer_id: "tr-3".to_string(),
                message_type: "transfer_notice".to_string(),
                direction: "outbound".to_string(),
                attempt_count: 12,
                created_at: 1,
                updated_at: 1,
            },
        );

        reconcile_once(&db, RetryPolicy::default(), 10).unwrap();
        reconcile_once(&db, RetryPolicy::default(), 20).unwrap();

        let r = db.read().unwrap();
        let messages = r.open_table(CROSS_NODE_MESSAGES).unwrap();
        let msg = messages.get("mid-3").unwrap().unwrap().value();
        assert_eq!(msg.direction, "terminal");

        let audits = r.open_table(TRANSFER_AUDIT_LOG).unwrap();
        let mut manual_review_count = 0;
        for row in audits.iter().unwrap() {
            let (_k, v) = row.unwrap();
            let evt = v.value();
            if evt.transfer_id == "tr-3" && evt.event_type == "reconciler.manual_review" {
                manual_review_count += 1;
            }
        }
        assert_eq!(manual_review_count, 1);
    }

    proptest! {
        #[test]
        fn prop_retry_delay_is_deterministic_and_non_decreasing(
            max_attempt in 1u32..=12,
        ) {
            let policy = RetryPolicy::default();
            let mut previous = 0u64;

            for attempt in 1..=max_attempt {
                let msg = CrossNodeMessageRecord {
                    message_id: "mid".to_string(),
                    transfer_id: "tr".to_string(),
                    message_type: "transfer_notice".to_string(),
                    direction: "outbound".to_string(),
                    attempt_count: attempt,
                    created_at: 1,
                    updated_at: 1,
                };

                let a = next_retry_delay_secs(&msg, policy);
                let b = next_retry_delay_secs(&msg, policy);
                prop_assert_eq!(a, b);
                prop_assert!(a >= previous);

                previous = a;
                prop_assert!(a <= policy.max_backoff_secs + (policy.max_backoff_secs / 4).max(1));
            }
        }
    }
}
