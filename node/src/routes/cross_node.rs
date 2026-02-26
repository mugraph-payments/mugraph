use mugraph_core::{
    error::Error,
    types::{
        Response, TransferChainState, TransferCreditState, TransferSettlementState,
        TransferStatusPayload, XNodeEnvelope, XNodeMessageType, validate_version,
    },
};

fn validate_envelope<T>(
    envelope: &XNodeEnvelope<T>,
    expected_message_type: XNodeMessageType,
) -> Result<(), Error> {
    if envelope.m != "xnode" {
        return Err(Error::InvalidInput {
            reason: "xnode envelope discriminator (m) must be 'xnode'".to_string(),
        });
    }

    validate_version(&envelope.version, 3).map_err(Error::from)?;

    if envelope.message_type != expected_message_type {
        return Err(Error::UnsupportedMessageType {
            message_type: format!(
                "expected {:?}, got {:?}",
                expected_message_type, envelope.message_type
            ),
        });
    }

    Ok(())
}

pub fn handle_create(
    request: &XNodeEnvelope<mugraph_core::types::TransferInitPayload>,
) -> Result<Response, Error> {
    validate_envelope(request, XNodeMessageType::TransferInit)?;

    Ok(Response::CrossNodeTransferCreate {
        transfer_id: request.transfer_id.clone(),
        accepted: true,
    })
}

pub fn handle_notify(
    request: &XNodeEnvelope<mugraph_core::types::TransferNoticePayload>,
) -> Result<Response, Error> {
    validate_envelope(request, XNodeMessageType::TransferNotice)?;

    Ok(Response::CrossNodeTransferNotify { accepted: true })
}

pub fn handle_status(
    request: &XNodeEnvelope<mugraph_core::types::TransferStatusQueryPayload>,
) -> Result<Response, Error> {
    validate_envelope(request, XNodeMessageType::TransferStatusQuery)?;

    Ok(Response::CrossNodeTransferStatus(XNodeEnvelope {
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
        payload: TransferStatusPayload {
            source_state: "requested".to_string(),
            destination_state: "notice_received".to_string(),
            settlement_state: TransferSettlementState::Submitted,
            chain_state: TransferChainState::Submitted,
            credit_state: TransferCreditState::None,
            tx_hash: None,
            confirmations_observed: 0,
            updated_at: request.sent_at.clone(),
        },
        auth: request.auth.clone(),
    }))
}

pub fn handle_ack(
    request: &XNodeEnvelope<mugraph_core::types::TransferAckPayload>,
) -> Result<Response, Error> {
    validate_envelope(request, XNodeMessageType::TransferAck)?;

    Ok(Response::CrossNodeTransferAck { accepted: true })
}

#[cfg(test)]
mod tests {
    use mugraph_core::types::{
        TransferAckPayload, TransferAckStatus, TransferInitPayload, TransferNoticePayload,
        TransferNoticeStage, TransferQueryType, TransferStatusQueryPayload, XNodeAuth,
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
}
