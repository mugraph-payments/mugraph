use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum XNodeMessageType {
    TransferInit,
    TransferNotice,
    TransferStatusQuery,
    TransferStatus,
    TransferAck,
}

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Eq)]
pub struct XNodeAuth {
    pub alg: String,
    pub kid: String,
    pub sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Eq)]
pub struct XNodeEnvelope<T> {
    pub m: String,
    pub version: String,
    pub message_type: XNodeMessageType,
    pub message_id: String,
    pub transfer_id: String,
    pub idempotency_key: String,
    pub correlation_id: String,
    pub origin_node_id: String,
    pub destination_node_id: String,
    pub sent_at: String,
    pub expires_at: Option<String>,
    pub payload: T,
    pub auth: XNodeAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Eq)]
pub struct TransferInitPayload {
    pub asset: String,
    pub amount: String,
    pub destination_account_ref: String,
    pub source_intent_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransferNoticeStage {
    Submitted,
    Confirmed,
    Finalized,
}

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Eq)]
pub struct TransferNoticePayload {
    pub notice_stage: TransferNoticeStage,
    pub tx_hash: String,
    pub confirmations: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransferQueryType {
    Current,
    History,
}

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Eq)]
pub struct TransferStatusQueryPayload {
    pub query_type: TransferQueryType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransferSettlementState {
    NotSubmitted,
    Submitted,
    Confirming,
    Confirmed,
    Invalidated,
    ManualReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransferChainState {
    Unknown,
    Submitted,
    Confirming,
    Confirmed,
    Invalidated,
}

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransferCreditState {
    None,
    Eligible,
    Credited,
    Held,
    Reversed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Eq)]
pub struct TransferStatusPayload {
    pub source_state: String,
    pub destination_state: String,
    pub settlement_state: TransferSettlementState,
    pub chain_state: TransferChainState,
    pub credit_state: TransferCreditState,
    pub tx_hash: Option<String>,
    pub confirmations_observed: u32,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransferAckStatus {
    Processed,
    Duplicate,
    Deferred,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary, PartialEq, Eq)]
pub struct TransferAckPayload {
    pub ack_for_message_id: String,
    pub ack_status: TransferAckStatus,
    pub ack_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum XNodeProtocolErrorCode {
    UnsupportedVersion,
    UnsupportedMessageType,
    SchemaValidationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct XNodeProtocolError {
    pub code: XNodeProtocolErrorCode,
    pub detail: String,
}

pub fn validate_version(
    version: &str,
    supported_major: u16,
) -> Result<(), XNodeProtocolError> {
    let Some((major, _minor)) = version.split_once('.') else {
        return Err(XNodeProtocolError {
            code: XNodeProtocolErrorCode::UnsupportedVersion,
            detail: format!("invalid version format: {version}"),
        });
    };

    let Ok(major) = major.parse::<u16>() else {
        return Err(XNodeProtocolError {
            code: XNodeProtocolErrorCode::UnsupportedVersion,
            detail: format!("invalid major version: {version}"),
        });
    };

    if major != supported_major {
        return Err(XNodeProtocolError {
            code: XNodeProtocolErrorCode::UnsupportedVersion,
            detail: format!("expected major {supported_major}, got {version}"),
        });
    }

    Ok(())
}

pub fn parse_message_type(
    value: &str,
) -> Result<XNodeMessageType, XNodeProtocolError> {
    match value {
        "transfer_init" => Ok(XNodeMessageType::TransferInit),
        "transfer_notice" => Ok(XNodeMessageType::TransferNotice),
        "transfer_status_query" => Ok(XNodeMessageType::TransferStatusQuery),
        "transfer_status" => Ok(XNodeMessageType::TransferStatus),
        "transfer_ack" => Ok(XNodeMessageType::TransferAck),
        _ => Err(XNodeProtocolError {
            code: XNodeProtocolErrorCode::UnsupportedMessageType,
            detail: format!("unsupported message type: {value}"),
        }),
    }
}

pub fn validate_envelope_basics<T>(
    envelope: &XNodeEnvelope<T>,
    expected_message_type: XNodeMessageType,
    supported_major: u16,
) -> Result<(), XNodeProtocolError> {
    if envelope.m != "xnode" {
        return Err(XNodeProtocolError {
            code: XNodeProtocolErrorCode::SchemaValidationFailed,
            detail: "xnode envelope discriminator (m) must be 'xnode'"
                .to_string(),
        });
    }

    validate_version(&envelope.version, supported_major)?;

    if envelope.message_type != expected_message_type {
        return Err(XNodeProtocolError {
            code: XNodeProtocolErrorCode::UnsupportedMessageType,
            detail: format!(
                "expected {:?}, got {:?}",
                expected_message_type, envelope.message_type
            ),
        });
    }

    let requires_expiry = matches!(
        envelope.message_type,
        XNodeMessageType::TransferInit
            | XNodeMessageType::TransferNotice
            | XNodeMessageType::TransferAck
    );

    if requires_expiry && envelope.expires_at.is_none() {
        return Err(XNodeProtocolError {
            code: XNodeProtocolErrorCode::SchemaValidationFailed,
            detail: "expires_at is required for command envelopes".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn validate_version_rejects_unsupported_major() {
        let result = validate_version("4.0", 3);
        assert_eq!(
            result.map_err(|e| e.code),
            Err(XNodeProtocolErrorCode::UnsupportedVersion)
        );
    }

    #[test]
    fn parse_message_type_rejects_unknown() {
        let result = parse_message_type("transfer_foo");
        assert_eq!(
            result.map_err(|e| e.code),
            Err(XNodeProtocolErrorCode::UnsupportedMessageType)
        );
    }

    #[test]
    fn transfer_status_payload_serializes_with_chain_and_credit_states() {
        let payload = TransferStatusPayload {
            source_state: "confirming".to_string(),
            destination_state: "credit_eligible".to_string(),
            settlement_state: TransferSettlementState::Confirming,
            chain_state: TransferChainState::Confirming,
            credit_state: TransferCreditState::Eligible,
            tx_hash: Some("abcd".to_string()),
            confirmations_observed: 5,
            updated_at: "2026-02-26T18:00:00Z".to_string(),
        };

        let value = serde_json::to_value(&payload).unwrap();
        assert_eq!(value["chain_state"], "confirming");
        assert_eq!(value["credit_state"], "eligible");
    }

    #[test]
    fn validate_envelope_basics_rejects_non_xnode_discriminator() {
        let env = XNodeEnvelope {
            m: "rpc".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferInit,
            message_id: "m1".to_string(),
            transfer_id: "t1".to_string(),
            idempotency_key: "ik".to_string(),
            correlation_id: "c1".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: "2026-02-26T18:00:00Z".to_string(),
            expires_at: Some("2026-02-26T18:05:00Z".to_string()),
            payload: TransferInitPayload {
                asset: "lovelace".to_string(),
                amount: "1".to_string(),
                destination_account_ref: "acct".to_string(),
                source_intent_hash: "h".to_string(),
            },
            auth: XNodeAuth {
                alg: "Ed25519".to_string(),
                kid: "k1".to_string(),
                sig: "sig".to_string(),
            },
        };

        let err =
            validate_envelope_basics(&env, XNodeMessageType::TransferInit, 3)
                .unwrap_err();
        assert_eq!(err.code, XNodeProtocolErrorCode::SchemaValidationFailed);
    }

    #[test]
    fn validate_envelope_basics_rejects_missing_expiry_for_command() {
        let env = XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferNotice,
            message_id: "m1".to_string(),
            transfer_id: "t1".to_string(),
            idempotency_key: "ik".to_string(),
            correlation_id: "c1".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: "2026-02-26T18:00:00Z".to_string(),
            expires_at: None,
            payload: TransferNoticePayload {
                notice_stage: TransferNoticeStage::Submitted,
                tx_hash: "abcd".to_string(),
                confirmations: None,
            },
            auth: XNodeAuth {
                alg: "Ed25519".to_string(),
                kid: "k1".to_string(),
                sig: "sig".to_string(),
            },
        };

        let err =
            validate_envelope_basics(&env, XNodeMessageType::TransferNotice, 3)
                .unwrap_err();
        assert_eq!(err.code, XNodeProtocolErrorCode::SchemaValidationFailed);
    }

    #[test]
    fn validate_version_accepts_supported_minor() {
        let result = validate_version("3.42", 3);
        assert!(result.is_ok());
    }

    proptest! {
        #[test]
        fn prop_validate_version_accepts_any_minor(minor in 0u16..=u16::MAX) {
            let version = format!("3.{minor}");
            prop_assert!(validate_version(&version, 3).is_ok());
        }

        #[test]
        fn prop_validate_version_rejects_other_majors(
            major in prop_oneof![0u16..3, 4u16..=10],
            minor in 0u16..=1000
        ) {
            let version = format!("{major}.{minor}");
            let err = validate_version(&version, 3).unwrap_err();
            prop_assert_eq!(err.code, XNodeProtocolErrorCode::UnsupportedVersion);
        }

        #[test]
        fn prop_command_messages_require_expires_at(msg in 0u8..=2) {
            let message_type = match msg {
                0 => XNodeMessageType::TransferInit,
                1 => XNodeMessageType::TransferNotice,
                _ => XNodeMessageType::TransferAck,
            };

            let env = XNodeEnvelope {
                m: "xnode".to_string(),
                version: "3.0".to_string(),
                message_type: message_type.clone(),
                message_id: "m1".to_string(),
                transfer_id: "t1".to_string(),
                idempotency_key: "ik".to_string(),
                correlation_id: "c1".to_string(),
                origin_node_id: "node://a".to_string(),
                destination_node_id: "node://b".to_string(),
                sent_at: "2026-02-26T18:00:00Z".to_string(),
                expires_at: None,
                payload: (),
                auth: XNodeAuth {
                    alg: "Ed25519".to_string(),
                    kid: "k1".to_string(),
                    sig: "sig".to_string(),
                },
            };

            let err = validate_envelope_basics(&env, message_type, 3).unwrap_err();
            prop_assert_eq!(err.code, XNodeProtocolErrorCode::SchemaValidationFailed);
        }

        #[test]
        fn prop_non_command_messages_allow_missing_expires_at(msg in 0u8..=1) {
            let message_type = match msg {
                0 => XNodeMessageType::TransferStatusQuery,
                _ => XNodeMessageType::TransferStatus,
            };

            let env = XNodeEnvelope {
                m: "xnode".to_string(),
                version: "3.0".to_string(),
                message_type: message_type.clone(),
                message_id: "m1".to_string(),
                transfer_id: "t1".to_string(),
                idempotency_key: "ik".to_string(),
                correlation_id: "c1".to_string(),
                origin_node_id: "node://a".to_string(),
                destination_node_id: "node://b".to_string(),
                sent_at: "2026-02-26T18:00:00Z".to_string(),
                expires_at: None,
                payload: (),
                auth: XNodeAuth {
                    alg: "Ed25519".to_string(),
                    kid: "k1".to_string(),
                    sig: "sig".to_string(),
                },
            };

            prop_assert!(validate_envelope_basics(&env, message_type, 3).is_ok());
        }
    }
}
