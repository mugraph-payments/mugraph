use mugraph_core::types::{
    CrossNodeMessageRecord,
    CrossNodeTransferRecord,
    TransferAuditEvent,
};
use mugraph_node::{
    database::{
        CROSS_NODE_MESSAGES,
        CROSS_NODE_TRANSFERS,
        Database,
        TRANSFER_AUDIT_LOG,
    },
    lifecycle::{
        LifecycleEvent,
        SourceLaneState,
        TransferLifecycle,
        apply_retry_exhaustion_to_record,
        status_payload_from_record,
    },
    observability::reconstruct_transfer_timeline,
    provider::{TxSettlementState, evaluate_tx_observation},
    reconciler::{RetryPolicy, reconcile_once},
};
use proptest::prelude::*;
use redb::ReadableTable;

fn temp_db_path(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "mugraph-m3-verification-{tag}-{}.db",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn seed_transfer_and_message(
    db: &Database,
    transfer_id: &str,
    message_id: &str,
    message_type: &str,
    attempt_count: u32,
) {
    let w = db.write().unwrap();
    {
        let mut transfers = w.open_table(CROSS_NODE_TRANSFERS).unwrap();
        transfers
            .insert(
                transfer_id,
                &CrossNodeTransferRecord {
                    transfer_id: transfer_id.to_string(),
                    source_node_id: "node://a".to_string(),
                    destination_node_id: "node://b".to_string(),
                    tx_hash: Some("txhash".to_string()),
                    chain_state: "confirming".to_string(),
                    credit_state: "none".to_string(),
                    confirmations_observed: 2,
                    created_at: 1,
                    updated_at: 1,
                },
            )
            .unwrap();
    }
    {
        let mut messages = w.open_table(CROSS_NODE_MESSAGES).unwrap();
        messages
            .insert(
                message_id,
                &CrossNodeMessageRecord {
                    message_id: message_id.to_string(),
                    transfer_id: transfer_id.to_string(),
                    message_type: message_type.to_string(),
                    direction: "outbound".to_string(),
                    attempt_count,
                    created_at: 1,
                    updated_at: 1,
                },
            )
            .unwrap();
    }
    w.commit().unwrap();
}

// M3-OBS-03/M3-OBS-08: retries are measurable and retry exhaustion converges deterministically.
#[test]
fn restart_recovery_is_idempotent_and_converges_to_manual_review() {
    let path = temp_db_path("restart");
    let db = Database::setup(&path).unwrap();
    db.migrate().unwrap();

    seed_transfer_and_message(&db, "tr-1", "mid-1", "transfer_notice", 1);

    // First pass schedules retries and persists progress.
    reconcile_once(&db, RetryPolicy::default(), 10_000).unwrap();

    // Simulate crash/restart by reopening DB.
    drop(db);
    let db_restarted = Database::setup(&path).unwrap();
    db_restarted.migrate().unwrap();

    // Run enough passes to force exhaustion path.
    for i in 0..20u64 {
        reconcile_once(
            &db_restarted,
            RetryPolicy::default(),
            10_000 + (i * 1_000),
        )
        .unwrap();
    }

    let r = db_restarted.read().unwrap();
    let transfers = r.open_table(CROSS_NODE_TRANSFERS).unwrap();
    let transfer = transfers.get("tr-1").unwrap().unwrap().value();
    assert_eq!(transfer.credit_state, "held");
    assert_eq!(transfer.chain_state, "invalidated");

    let audits = r.open_table(TRANSFER_AUDIT_LOG).unwrap();
    let mut saw_manual_review = false;
    for row in audits.iter().unwrap() {
        let (_k, v) = row.unwrap();
        let event = v.value();
        if event.transfer_id == "tr-1"
            && event.event_type == "reconciler.manual_review"
        {
            saw_manual_review = true;
            break;
        }
    }
    assert!(saw_manual_review);
}

#[test]
fn shared_helpers_preserve_manual_review_mapping() {
    let mut record = CrossNodeTransferRecord {
        transfer_id: "tr-helper".to_string(),
        source_node_id: "node://a".to_string(),
        destination_node_id: "node://b".to_string(),
        tx_hash: Some("txhash".to_string()),
        chain_state: "confirming".to_string(),
        credit_state: "none".to_string(),
        confirmations_observed: 2,
        created_at: 1,
        updated_at: 2,
    };

    apply_retry_exhaustion_to_record(&mut record);
    let payload = status_payload_from_record(&record);

    assert_eq!(record.chain_state, "invalidated");
    assert_eq!(record.credit_state, "held");
    assert_eq!(
        payload.chain_state,
        mugraph_core::types::TransferChainState::Invalidated
    );
    assert_eq!(
        payload.credit_state,
        mugraph_core::types::TransferCreditState::Held
    );
    assert_eq!(
        payload.settlement_state,
        mugraph_core::types::TransferSettlementState::ManualReview
    );
}

// M3-SEC-05/M3-OBS-08: deep reorg invalidation is deterministic.
#[test]
fn deep_reorg_observation_forces_invalidated_path() {
    let mut lifecycle = TransferLifecycle::new();
    lifecycle.apply(LifecycleEvent::SourceSubmitted);
    lifecycle.apply(LifecycleEvent::ChainObserved {
        tx_hash: "tx1",
        confirmations: 12,
        confirmed: true,
    });

    let obs = evaluate_tx_observation("tx1", None, 1_000, 12, 6, true);
    assert_eq!(obs.state, TxSettlementState::Invalidated);

    lifecycle.apply(LifecycleEvent::ChainInvalidated);
    assert_eq!(lifecycle.source, SourceLaneState::Invalidated);
}

#[derive(Clone, Debug)]
enum DeliveryOp {
    Notice,
    Ack,
    Confirm,
    Credit,
    Invalidate,
}

fn delivery_ops() -> impl Strategy<Value = Vec<DeliveryOp>> {
    prop::collection::vec(
        prop_oneof![
            Just(DeliveryOp::Notice),
            Just(DeliveryOp::Ack),
            Just(DeliveryOp::Confirm),
            Just(DeliveryOp::Credit),
            Just(DeliveryOp::Invalidate),
        ],
        0..64,
    )
}

// M3-SEC-04: duplicate/reordered deliveries cannot create double-credit.
proptest! {
    #[test]
    fn prop_no_double_credit_under_duplicate_or_reordered_delivery(ops in delivery_ops()) {
        let mut lifecycle = TransferLifecycle::new();
        lifecycle.apply(LifecycleEvent::SourceSubmitted);

        let mut credit_transitions = 0u32;
        let mut was_credited = false;

        for op in ops {
            match op {
                DeliveryOp::Notice => lifecycle.apply(LifecycleEvent::DestinationNoticeReceived),
                DeliveryOp::Ack => lifecycle.apply(LifecycleEvent::AckReceived),
                DeliveryOp::Confirm => lifecycle.apply(LifecycleEvent::ChainObserved {
                    tx_hash: "tx1",
                    confirmations: 12,
                    confirmed: true,
                }),
                DeliveryOp::Credit => lifecycle.apply(LifecycleEvent::DestinationCredited),
                DeliveryOp::Invalidate => lifecycle.apply(LifecycleEvent::ChainInvalidated),
            }

            let now_credited = lifecycle.credit == mugraph_core::types::TransferCreditState::Credited;
            if !was_credited && now_credited {
                credit_transitions += 1;
            }
            was_credited = now_credited;
        }

        prop_assert!(credit_transitions <= 1);
    }
}

// M3-OBS-06: audit timeline reconstruction is deterministic.
#[test]
fn audit_timeline_reconstruction_orders_events_for_transfer() {
    let db = Database::setup(temp_db_path("timeline")).unwrap();
    db.migrate().unwrap();

    let w = db.write().unwrap();
    {
        let mut t = w.open_table(TRANSFER_AUDIT_LOG).unwrap();
        t.insert(
            "e3",
            &TransferAuditEvent {
                event_id: "e3".to_string(),
                transfer_id: "tr-2".to_string(),
                event_type: "transfer.notice.accepted".to_string(),
                reason: "notice".to_string(),
                created_at: 20,
            },
        )
        .unwrap();
        t.insert(
            "e1",
            &TransferAuditEvent {
                event_id: "e1".to_string(),
                transfer_id: "tr-2".to_string(),
                event_type: "transfer.initiated".to_string(),
                reason: "init".to_string(),
                created_at: 10,
            },
        )
        .unwrap();
        t.insert(
            "e2",
            &TransferAuditEvent {
                event_id: "e2".to_string(),
                transfer_id: "tr-2".to_string(),
                event_type: "transfer.confirmed".to_string(),
                reason: "confirmed".to_string(),
                created_at: 30,
            },
        )
        .unwrap();
    }
    w.commit().unwrap();

    let timeline = reconstruct_transfer_timeline(&db, "tr-2").unwrap();
    let kinds: Vec<_> = timeline.into_iter().map(|e| e.event_type).collect();
    assert_eq!(
        kinds,
        vec![
            "transfer.initiated",
            "transfer.notice.accepted",
            "transfer.confirmed"
        ]
    );
}
