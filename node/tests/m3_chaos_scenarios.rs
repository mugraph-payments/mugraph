use mugraph_core::types::{
    TransferChainState, TransferSettlementState, XNodeMessageType,
    validate_envelope_basics,
};
use mugraph_node::lifecycle::{
    DestinationLaneState, LifecycleEvent, TransferLifecycle,
};
use proptest::prelude::*;

#[derive(Clone, Debug)]
enum Op {
    DestinationDown,
    DestinationUp,
    SendNotice { delivered: bool },
    ObserveConfirmed,
    Credit,
    Ack,
}

fn op_schedule() -> impl Strategy<Value = Vec<Op>> {
    prop::collection::vec(
        prop_oneof![
            Just(Op::DestinationDown),
            Just(Op::DestinationUp),
            any::<bool>().prop_map(|delivered| Op::SendNotice { delivered }),
            Just(Op::ObserveConfirmed),
            Just(Op::Credit),
            Just(Op::Ack),
        ],
        0..64,
    )
}

// Multi-node chaos scenario: destination downtime + packet loss/timeouts converge after recovery.
#[test]
fn downtime_and_packet_loss_eventually_converge_after_recovery() {
    let mut lifecycle = TransferLifecycle::new();
    lifecycle.apply(LifecycleEvent::SourceSubmitted);

    let mut destination_up = false;
    let mut pending_notice = false;

    // Simulate retries with losses while destination is down.
    for delivered in [false, false, true] {
        if delivered && destination_up {
            lifecycle.apply(LifecycleEvent::DestinationNoticeReceived);
        } else if delivered {
            pending_notice = true;
        }
    }

    // Destination recovers and consumes pending notice.
    destination_up = true;
    if destination_up && pending_notice {
        lifecycle.apply(LifecycleEvent::DestinationNoticeReceived);
    }

    lifecycle.apply(LifecycleEvent::ChainObserved {
        tx_hash: "tx-chaos",
        confirmations: 12,
        confirmed: true,
    });
    lifecycle.apply(LifecycleEvent::DestinationCredited);

    assert_eq!(lifecycle.chain, TransferChainState::Confirmed);
    assert_eq!(lifecycle.settlement, TransferSettlementState::Confirmed);
    assert_eq!(lifecycle.destination, DestinationLaneState::Credited);
}

// Mixed-version compatibility: unsupported major should be rejected deterministically.
#[test]
fn mixed_version_scenario_rejects_unsupported_major() {
    let envelope = mugraph_core::types::XNodeEnvelope {
        m: "xnode".to_string(),
        version: "2.9".to_string(),
        message_type: XNodeMessageType::TransferNotice,
        message_id: "mid".to_string(),
        transfer_id: "tr".to_string(),
        idempotency_key: "ik".to_string(),
        correlation_id: "corr".to_string(),
        origin_node_id: "node://a".to_string(),
        destination_node_id: "node://b".to_string(),
        sent_at: "2026-02-26T18:00:00Z".to_string(),
        expires_at: Some("2026-02-26T18:05:00Z".to_string()),
        payload: (),
        auth: mugraph_core::types::XNodeAuth {
            alg: "Ed25519".to_string(),
            kid: "k1".to_string(),
            sig: "sig".to_string(),
        },
    };

    let err = validate_envelope_basics(
        &envelope,
        XNodeMessageType::TransferNotice,
        3,
    )
    .unwrap_err();
    assert_eq!(
        err.code,
        mugraph_core::types::XNodeProtocolErrorCode::UnsupportedVersion
    );
}

proptest! {
    // Chaos invariant: no double credit and stale ack does not regress terminal confirmation.
    #[test]
    fn prop_chaos_schedule_preserves_credit_and_ack_invariants(ops in op_schedule()) {
        let mut lifecycle = TransferLifecycle::new();
        lifecycle.apply(LifecycleEvent::SourceSubmitted);

        let mut destination_up = true;
        let mut pending_notice = false;
        let mut credit_transitions = 0u32;
        let mut was_credited = false;

        for op in ops {
            let before = lifecycle.clone();
            match op {
                Op::DestinationDown => destination_up = false,
                Op::DestinationUp => {
                    destination_up = true;
                    if pending_notice {
                        lifecycle.apply(LifecycleEvent::DestinationNoticeReceived);
                        pending_notice = false;
                    }
                }
                Op::SendNotice { delivered } => {
                    if delivered && destination_up {
                        lifecycle.apply(LifecycleEvent::DestinationNoticeReceived);
                    } else if delivered {
                        pending_notice = true;
                    }
                }
                Op::ObserveConfirmed => lifecycle.apply(LifecycleEvent::ChainObserved {
                    tx_hash: "tx-chaos",
                    confirmations: 12,
                    confirmed: true,
                }),
                Op::Credit => lifecycle.apply(LifecycleEvent::DestinationCredited),
                Op::Ack => lifecycle.apply(LifecycleEvent::AckReceived),
            }

            let now_credited = lifecycle.credit == mugraph_core::types::TransferCreditState::Credited;
            if !was_credited && now_credited {
                credit_transitions += 1;
            }
            was_credited = now_credited;

            if matches!(op, Op::Ack)
                && before.source == mugraph_node::lifecycle::SourceLaneState::Confirmed
                && before.chain == TransferChainState::Confirmed
            {
                prop_assert_eq!(lifecycle.clone(), before);
            }
        }

        prop_assert!(credit_transitions <= 1);
    }
}
