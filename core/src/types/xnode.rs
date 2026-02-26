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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct XNodeProtocolError {
    pub code: XNodeProtocolErrorCode,
    pub detail: String,
}

pub fn validate_version(version: &str, supported_major: u16) -> Result<(), XNodeProtocolError> {
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

pub fn parse_message_type(value: &str) -> Result<XNodeMessageType, XNodeProtocolError> {
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

#[cfg(test)]
mod tests {
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
}
