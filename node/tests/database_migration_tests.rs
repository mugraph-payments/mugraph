use mugraph_core::types::{
    CrossNodeMessageRecord, CrossNodeTransferRecord, IdempotencyRecord,
    TransferAuditEvent,
};
use mugraph_node::database::{
    CROSS_NODE_MESSAGES, CROSS_NODE_TRANSFERS, Database, IDEMPOTENCY_KEYS,
    TRANSFER_AUDIT_LOG,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn temp_db_path() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "mugraph-db-migration-{}.db",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn migrates_schema_to_v3() -> TestResult {
    let db = Database::setup(temp_db_path())?;
    db.migrate()?;
    assert_eq!(db.schema_version()?, 3);
    Ok(())
}

#[test]
fn cross_node_tables_support_read_write() -> TestResult {
    let db = Database::setup(temp_db_path())?;
    db.migrate()?;

    let transfer = CrossNodeTransferRecord {
        transfer_id: "tr-1".to_string(),
        source_node_id: "node://a".to_string(),
        destination_node_id: "node://b".to_string(),
        tx_hash: Some("abcd".to_string()),
        chain_state: "submitted".to_string(),
        credit_state: "none".to_string(),
        confirmations_observed: 0,
        created_at: 1,
        updated_at: 1,
    };

    let message = CrossNodeMessageRecord {
        message_id: "mid-1".to_string(),
        transfer_id: "tr-1".to_string(),
        message_type: "transfer_notice".to_string(),
        direction: "outbound".to_string(),
        attempt_count: 1,
        created_at: 1,
        updated_at: 1,
    };

    let idem = IdempotencyRecord {
        idempotency_key: "ik-1".to_string(),
        transfer_id: "tr-1".to_string(),
        message_type: "transfer_notice".to_string(),
        request_hash: "h".to_string(),
        first_seen_at: 1,
        expires_at: 2,
    };

    let audit = TransferAuditEvent {
        event_id: "evt-1".to_string(),
        transfer_id: "tr-1".to_string(),
        event_type: "transfer.notice.accepted".to_string(),
        reason: "ok".to_string(),
        created_at: 1,
    };

    let w = db.write()?;
    {
        let mut transfers = w.open_table(CROSS_NODE_TRANSFERS)?;
        transfers.insert("tr-1", &transfer)?;
    }
    {
        let mut messages = w.open_table(CROSS_NODE_MESSAGES)?;
        messages.insert("mid-1", &message)?;
    }
    {
        let mut idems = w.open_table(IDEMPOTENCY_KEYS)?;
        idems.insert("ik-1", &idem)?;
    }
    {
        let mut audits = w.open_table(TRANSFER_AUDIT_LOG)?;
        audits.insert("evt-1", &audit)?;
    }
    w.commit()?;

    let r = db.read()?;
    {
        let transfers = r.open_table(CROSS_NODE_TRANSFERS)?;
        let got = transfers.get("tr-1")?.unwrap().value();
        assert_eq!(got.transfer_id, "tr-1");
        assert_eq!(got.chain_state, "submitted");
    }
    {
        let messages = r.open_table(CROSS_NODE_MESSAGES)?;
        let got = messages.get("mid-1")?.unwrap().value();
        assert_eq!(got.transfer_id, "tr-1");
    }
    {
        let idems = r.open_table(IDEMPOTENCY_KEYS)?;
        let got = idems.get("ik-1")?.unwrap().value();
        assert_eq!(got.transfer_id, "tr-1");
    }
    {
        let audits = r.open_table(TRANSFER_AUDIT_LOG)?;
        let got = audits.get("evt-1")?.unwrap().value();
        assert_eq!(got.event_type, "transfer.notice.accepted");
    }

    Ok(())
}
