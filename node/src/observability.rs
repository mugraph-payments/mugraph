use mugraph_core::{error::Error, types::TransferAuditEvent};
use redb::ReadableTable;

use crate::database::{Database, TRANSFER_AUDIT_LOG};

pub fn reconstruct_transfer_timeline(
    database: &Database,
    transfer_id: &str,
) -> Result<Vec<TransferAuditEvent>, Error> {
    let read_tx = database.read()?;
    let audits = read_tx.open_table(TRANSFER_AUDIT_LOG)?;

    let mut events = Vec::new();
    for row in audits.iter()? {
        let (_k, v) = row?;
        let evt = v.value();
        if evt.transfer_id == transfer_id {
            events.push(evt);
        }
    }

    events.sort_by(|a, b| {
        a.created_at
            .cmp(&b.created_at)
            .then_with(|| a.event_id.cmp(&b.event_id))
    });

    Ok(events)
}

#[cfg(test)]
mod tests {
    use mugraph_core::types::TransferAuditEvent;

    use super::*;
    use crate::database::TRANSFER_AUDIT_LOG;

    fn temp_db() -> Database {
        let path = std::env::temp_dir().join(format!(
            "mugraph-observability-test-{}.db",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let db = Database::setup(path).unwrap();
        db.migrate().unwrap();
        db
    }

    #[test]
    fn reconstructs_ordered_timeline_for_transfer() {
        let db = temp_db();

        let w = db.write().unwrap();
        {
            let mut t = w.open_table(TRANSFER_AUDIT_LOG).unwrap();
            t.insert(
                "3",
                &TransferAuditEvent {
                    event_id: "3".to_string(),
                    transfer_id: "tr-1".to_string(),
                    event_type: "transfer.credited".to_string(),
                    reason: "credited".to_string(),
                    created_at: 30,
                },
            )
            .unwrap();
            t.insert(
                "1",
                &TransferAuditEvent {
                    event_id: "1".to_string(),
                    transfer_id: "tr-1".to_string(),
                    event_type: "transfer.initiated".to_string(),
                    reason: "init".to_string(),
                    created_at: 10,
                },
            )
            .unwrap();
            t.insert(
                "2",
                &TransferAuditEvent {
                    event_id: "2".to_string(),
                    transfer_id: "tr-1".to_string(),
                    event_type: "transfer.notice.accepted".to_string(),
                    reason: "notice".to_string(),
                    created_at: 20,
                },
            )
            .unwrap();
            t.insert(
                "x",
                &TransferAuditEvent {
                    event_id: "x".to_string(),
                    transfer_id: "tr-2".to_string(),
                    event_type: "transfer.initiated".to_string(),
                    reason: "other".to_string(),
                    created_at: 1,
                },
            )
            .unwrap();
        }
        w.commit().unwrap();

        let timeline = reconstruct_transfer_timeline(&db, "tr-1").unwrap();
        assert_eq!(timeline.len(), 3);
        assert_eq!(timeline[0].event_type, "transfer.initiated");
        assert_eq!(timeline[1].event_type, "transfer.notice.accepted");
        assert_eq!(timeline[2].event_type, "transfer.credited");
    }
}
