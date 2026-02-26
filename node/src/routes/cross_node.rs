use mugraph_core::{
    error::Error,
    types::{
        Response, XNodeEnvelope, XNodeMessageType, validate_envelope_basics,
    },
};

use crate::lifecycle::{LifecycleEvent, TransferLifecycle};

pub fn handle_create(
    request: &XNodeEnvelope<mugraph_core::types::TransferInitPayload>,
) -> Result<Response, Error> {
    validate_envelope_basics(request, XNodeMessageType::TransferInit, 3).map_err(Error::from)?;

    Ok(Response::CrossNodeTransferCreate {
        transfer_id: request.transfer_id.clone(),
        accepted: true,
    })
}

pub fn handle_notify(
    request: &XNodeEnvelope<mugraph_core::types::TransferNoticePayload>,
) -> Result<Response, Error> {
    validate_envelope_basics(request, XNodeMessageType::TransferNotice, 3).map_err(Error::from)?;

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
) -> Result<Response, Error> {
    validate_envelope_basics(request, XNodeMessageType::TransferAck, 3).map_err(Error::from)?;

    Ok(Response::CrossNodeTransferAck { accepted: true })
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use mugraph_core::types::{
        TransferAckPayload, TransferAckStatus, TransferChainState, TransferCreditState,
        TransferInitPayload, TransferNoticePayload, TransferNoticeStage, TransferQueryType,
        TransferStatusQueryPayload, XNodeAuth,
    };

    use super::*;

    fn auth() -> XNodeAuth {
        XNodeAuth {
            alg: "Ed25519".to_string(),
            kid: "k1".to_string(),
            sig: "sig".to_string(),
        }
    }

    #[test]
    fn create_rejects_unsupported_version() {
        let request = XNodeEnvelope {
            m: "xnode".to_string(),
            version: "4.0".to_string(),
            message_type: XNodeMessageType::TransferInit,
            message_id: "mid".to_string(),
            transfer_id: "tr".to_string(),
            idempotency_key: "ik".to_string(),
            correlation_id: "corr".to_string(),
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
        };

        let err = handle_create(&request).unwrap_err();
        assert!(matches!(err, Error::UnsupportedVersion { .. }));
    }

    #[test]
    fn notify_rejects_wrong_message_type() {
        let request = XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferInit,
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

        let err = handle_notify(&request).unwrap_err();
        assert!(matches!(err, Error::UnsupportedMessageType { .. }));
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
        let request = XNodeEnvelope {
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

        let response = handle_ack(&request).unwrap();
        assert!(matches!(
            response,
            Response::CrossNodeTransferAck { accepted: true }
        ));
    }

    #[test]
    fn create_rejects_non_xnode_discriminator() {
        let request = XNodeEnvelope {
            m: "rpc".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferInit,
            message_id: "mid".to_string(),
            transfer_id: "tr".to_string(),
            idempotency_key: "ik".to_string(),
            correlation_id: "corr".to_string(),
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
        };

        let err = handle_create(&request).unwrap_err();
        assert!(matches!(err, Error::InvalidInput { .. }));
    }

    #[test]
    fn notice_rejects_missing_expires_at() {
        let request = XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferNotice,
            message_id: "mid".to_string(),
            transfer_id: "tr".to_string(),
            idempotency_key: "ik".to_string(),
            correlation_id: "corr".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: "2026-02-26T18:00:00Z".to_string(),
            expires_at: None,
            payload: TransferNoticePayload {
                notice_stage: TransferNoticeStage::Confirmed,
                tx_hash: "abcd".to_string(),
                confirmations: Some(1),
            },
            auth: auth(),
        };

        let err = handle_notify(&request).unwrap_err();
        assert!(matches!(err, Error::InvalidInput { .. }));
    }

    #[test]
    fn create_rejects_missing_expires_at() {
        let request = XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferInit,
            message_id: "mid".to_string(),
            transfer_id: "tr".to_string(),
            idempotency_key: "ik".to_string(),
            correlation_id: "corr".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: "2026-02-26T18:00:00Z".to_string(),
            expires_at: None,
            payload: TransferInitPayload {
                asset: "lovelace".to_string(),
                amount: "1".to_string(),
                destination_account_ref: "acct".to_string(),
                source_intent_hash: "hash".to_string(),
            },
            auth: auth(),
        };

        let err = handle_create(&request).unwrap_err();
        assert!(matches!(err, Error::InvalidInput { .. }));
    }

    #[test]
    fn ack_rejects_missing_expires_at() {
        let request = XNodeEnvelope {
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
            expires_at: None,
            payload: TransferAckPayload {
                ack_for_message_id: "mid2".to_string(),
                ack_status: TransferAckStatus::Processed,
                ack_at: "2026-02-26T18:00:01Z".to_string(),
            },
            auth: auth(),
        };

        let err = handle_ack(&request).unwrap_err();
        assert!(matches!(err, Error::InvalidInput { .. }));
    }

    proptest! {
        #[test]
        fn prop_duplicate_notify_is_idempotent(
            tx_hash in "[a-f0-9]{1,32}",
            confirmations in 0u32..=100,
        ) {
            let request = XNodeEnvelope {
                m: "xnode".to_string(),
                version: "3.0".to_string(),
                message_type: XNodeMessageType::TransferNotice,
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
                    tx_hash,
                    confirmations: Some(confirmations),
                },
                auth: auth(),
            };

            let first = handle_notify(&request).unwrap();
            let second = handle_notify(&request).unwrap();

            let first_ok = matches!(first, Response::CrossNodeTransferNotify { accepted: true });
            let second_ok = matches!(second, Response::CrossNodeTransferNotify { accepted: true });
            prop_assert!(first_ok);
            prop_assert!(second_ok);
        }

        #[test]
        fn prop_duplicate_create_preserves_transfer_identity(
            transfer_id in "tr_[a-z0-9]{3,20}",
            amount in 1u64..=1_000_000,
        ) {
            let request = XNodeEnvelope {
                m: "xnode".to_string(),
                version: "3.0".to_string(),
                message_type: XNodeMessageType::TransferInit,
                message_id: "mid".to_string(),
                transfer_id: transfer_id.clone(),
                idempotency_key: "ik".to_string(),
                correlation_id: "corr".to_string(),
                origin_node_id: "node://a".to_string(),
                destination_node_id: "node://b".to_string(),
                sent_at: "2026-02-26T18:00:00Z".to_string(),
                expires_at: Some("2026-02-26T18:05:00Z".to_string()),
                payload: TransferInitPayload {
                    asset: "lovelace".to_string(),
                    amount: amount.to_string(),
                    destination_account_ref: "acct".to_string(),
                    source_intent_hash: "hash".to_string(),
                },
                auth: auth(),
            };

            let first = handle_create(&request).unwrap();
            let second = handle_create(&request).unwrap();

            match (first, second) {
                (
                    Response::CrossNodeTransferCreate { transfer_id: t1, accepted: a1 },
                    Response::CrossNodeTransferCreate { transfer_id: t2, accepted: a2 }
                ) => {
                    prop_assert_eq!(t1, transfer_id.clone());
                    prop_assert_eq!(t2, transfer_id.clone());
                    prop_assert!(a1 && a2);
                }
                _ => prop_assert!(false, "unexpected response variant"),
            }
        }

        #[test]
        fn prop_stale_ack_does_not_regress_status(
            ack_for in "mid_[a-z0-9]{1,12}",
            ack_at in "2026-02-26T18:[0-5][0-9]:[0-5][0-9]Z",
        ) {
            let status_query = XNodeEnvelope {
                m: "xnode".to_string(),
                version: "3.0".to_string(),
                message_type: XNodeMessageType::TransferStatusQuery,
                message_id: "mid-status".to_string(),
                transfer_id: "tr".to_string(),
                idempotency_key: "ik-status".to_string(),
                correlation_id: "corr".to_string(),
                origin_node_id: "node://a".to_string(),
                destination_node_id: "node://b".to_string(),
                sent_at: "2026-02-26T18:00:00Z".to_string(),
                expires_at: None,
                payload: TransferStatusQueryPayload { query_type: TransferQueryType::Current },
                auth: auth(),
            };

            let before = handle_status(&status_query).unwrap();

            let ack = XNodeEnvelope {
                m: "xnode".to_string(),
                version: "3.0".to_string(),
                message_type: XNodeMessageType::TransferAck,
                message_id: "mid-ack".to_string(),
                transfer_id: "tr".to_string(),
                idempotency_key: "ik-ack".to_string(),
                correlation_id: "corr".to_string(),
                origin_node_id: "node://a".to_string(),
                destination_node_id: "node://b".to_string(),
                sent_at: "2026-02-26T18:00:00Z".to_string(),
                expires_at: Some("2026-02-26T18:05:00Z".to_string()),
                payload: TransferAckPayload {
                    ack_for_message_id: ack_for,
                    ack_status: TransferAckStatus::Processed,
                    ack_at,
                },
                auth: auth(),
            };

            let _ = handle_ack(&ack).unwrap();

            let after = handle_status(&status_query).unwrap();

            match (before, after) {
                (Response::CrossNodeTransferStatus(b), Response::CrossNodeTransferStatus(a)) => {
                    prop_assert_eq!(b.payload.settlement_state, a.payload.settlement_state);
                    prop_assert_eq!(b.payload.chain_state, a.payload.chain_state);
                    prop_assert_eq!(b.payload.credit_state, a.payload.credit_state);
                }
                _ => prop_assert!(false, "unexpected response variant"),
            }
        }

        #[test]
        fn prop_handlers_accept_supported_minor_versions(minor in 0u16..=2000) {
            let version = format!("3.{minor}");

            let create = XNodeEnvelope {
                m: "xnode".to_string(),
                version: version.clone(),
                message_type: XNodeMessageType::TransferInit,
                message_id: "mid-c".to_string(),
                transfer_id: "tr".to_string(),
                idempotency_key: "ik-c".to_string(),
                correlation_id: "corr".to_string(),
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
            };

            let status = XNodeEnvelope {
                m: "xnode".to_string(),
                version,
                message_type: XNodeMessageType::TransferStatusQuery,
                message_id: "mid-s".to_string(),
                transfer_id: "tr".to_string(),
                idempotency_key: "ik-s".to_string(),
                correlation_id: "corr".to_string(),
                origin_node_id: "node://a".to_string(),
                destination_node_id: "node://b".to_string(),
                sent_at: "2026-02-26T18:00:00Z".to_string(),
                expires_at: None,
                payload: TransferStatusQueryPayload { query_type: TransferQueryType::Current },
                auth: auth(),
            };

            prop_assert!(handle_create(&create).is_ok());
            prop_assert!(handle_status(&status).is_ok());
        }

        #[test]
        fn prop_notify_rejects_any_non_notice_message_type(mt in 0u8..=3) {
            let message_type = match mt {
                0 => XNodeMessageType::TransferInit,
                1 => XNodeMessageType::TransferStatusQuery,
                2 => XNodeMessageType::TransferStatus,
                _ => XNodeMessageType::TransferAck,
            };

            let request = XNodeEnvelope {
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

            let err = handle_notify(&request).unwrap_err();
            let is_unsupported = matches!(err, Error::UnsupportedMessageType { .. });
            prop_assert!(is_unsupported);
        }
    }
}
