use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::types::{
    AssetName,
    BlindSignature,
    PolicyId,
    Refresh,
    TransferAckPayload,
    TransferInitPayload,
    TransferNoticePayload,
    TransferStatusQueryPayload,
    XNodeEnvelope,
};

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
#[serde(tag = "m", content = "p")]
pub enum Request {
    #[serde(rename = "refresh")]
    Refresh(Refresh),
    #[serde(rename = "emit")]
    Emit {
        policy_id: PolicyId,
        asset_name: AssetName,
        amount: u64,
    },
    #[serde(rename = "public_key")]
    Info,
    #[serde(rename = "deposit")]
    Deposit(DepositRequest),
    #[serde(rename = "withdraw")]
    Withdraw(WithdrawRequest),
    #[serde(rename = "cross_node_transfer_create")]
    CrossNodeTransferCreate(XNodeEnvelope<TransferInitPayload>),
    #[serde(rename = "cross_node_transfer_notify")]
    CrossNodeTransferNotify(XNodeEnvelope<TransferNoticePayload>),
    #[serde(rename = "cross_node_transfer_status")]
    CrossNodeTransferStatus(XNodeEnvelope<TransferStatusQueryPayload>),
    #[serde(rename = "cross_node_transfer_ack")]
    CrossNodeTransferAck(XNodeEnvelope<TransferAckPayload>),
}

/// Deposit request from user
/// User sends funds to script address and provides proof of deposit
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
pub struct DepositRequest {
    /// UTxO reference (tx_hash + index) at the script address
    pub utxo: UtxoReference,
    /// Blinded outputs to mint
    pub outputs: Vec<BlindSignature>,
    /// Message signed by user (canonical JSON)
    pub message: String,
    /// CIP-8 signature over canonical payload
    pub signature: Vec<u8>,
    /// Nonce/timestamp to prevent replay
    pub nonce: u64,
    /// Network tag (mainnet/preprod/etc)
    pub network: String,
}

/// UTxO reference for deposits
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
pub struct UtxoReference {
    /// Transaction hash (hex encoded)
    pub tx_hash: String,
    /// Output index
    pub index: u16,
}

/// Withdrawal request from user
/// User provides unsigned transaction spending script UTxOs
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
pub struct WithdrawRequest {
    /// Notes to burn (blinded inputs)
    pub notes: Vec<BlindSignature>,
    /// Unsigned transaction CBOR (hex encoded)
    pub tx_cbor: String,
    /// Transaction hash (expected)
    pub tx_hash: String,
}

/// Deposit response
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
pub struct DepositResponse {
    /// Blind signatures for the outputs
    pub signatures: Vec<BlindSignature>,
    /// Deposit reference (UTxO identifier)
    pub deposit_ref: String,
}

/// Withdrawal response
#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
pub struct WithdrawResponse {
    /// Fully signed transaction CBOR (hex encoded)
    pub signed_tx_cbor: String,
    /// Transaction hash
    pub tx_hash: String,
    /// Change notes (if any)
    pub change_notes: Vec<BlindSignature>,
}

#[cfg(test)]
mod tests {
    use proptest::prop_assert_eq;
    use serde_json::Value;
    use test_strategy::proptest;

    use crate::types::{
        Refresh,
        Request,
        TransferAckPayload,
        TransferAckStatus,
        TransferInitPayload,
        TransferNoticePayload,
        TransferNoticeStage,
        TransferQueryType,
        TransferStatusQueryPayload,
        XNodeAuth,
        XNodeEnvelope,
        XNodeMessageType,
    };

    #[proptest]
    fn test_serde_roundtrip(request: Request) {
        let json = serde_json::to_string(&request).unwrap();
        let decoded: Request = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&decoded).unwrap();
        prop_assert_eq!(json, json2);
    }

    #[test]
    fn test_serialization() {
        let request: Request = Request::Refresh(Refresh::default());

        let expected = Value::Object({
            let mut map = serde_json::Map::new();
            map.insert("m".to_string(), Value::String("refresh".to_string()));
            map.insert(
                "p".to_string(),
                Value::Object({
                    let mut inner = serde_json::Map::new();
                    inner.insert("m".to_string(), Value::Number(0.into()));
                    inner.insert("a".to_string(), Value::Array(vec![]));
                    inner.insert("a_".to_string(), Value::Array(vec![]));
                    inner.insert("s".to_string(), Value::Array(vec![]));
                    inner
                }),
            );
            map
        });

        assert_eq!(expected, serde_json::to_value(&request).unwrap());
    }

    #[test]
    fn test_info_serialization() {
        let request: Request = Request::Info;
        let expected = Value::Object({
            let mut map = serde_json::Map::new();
            map.insert(
                "m".to_string(),
                Value::String("public_key".to_string()),
            );
            map
        });

        assert_eq!(expected, serde_json::to_value(&request).unwrap());
    }

    #[test]
    fn test_cross_node_transfer_notify_serialization() {
        let request = Request::CrossNodeTransferNotify(XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferNotice,
            message_id: "mid-1".to_string(),
            transfer_id: "tr-1".to_string(),
            idempotency_key: "ik-1".to_string(),
            correlation_id: "corr-1".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: "2026-02-26T18:00:00Z".to_string(),
            expires_at: Some("2026-02-26T18:05:00Z".to_string()),
            payload: TransferNoticePayload {
                notice_stage: TransferNoticeStage::Confirmed,
                tx_hash: "abcd".to_string(),
                confirmations: Some(6),
            },
            auth: XNodeAuth {
                alg: "Ed25519".to_string(),
                kid: "k1".to_string(),
                sig: "sig".to_string(),
            },
        });

        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["m"], "cross_node_transfer_notify");
        assert_eq!(value["p"]["message_type"], "transfer_notice");
        assert_eq!(value["p"]["payload"]["tx_hash"], "abcd");
    }

    #[test]
    fn test_cross_node_request_contract_shapes() {
        let auth = XNodeAuth {
            alg: "Ed25519".to_string(),
            kid: "k1".to_string(),
            sig: "sig".to_string(),
        };

        let create = Request::CrossNodeTransferCreate(XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.7".to_string(),
            message_type: XNodeMessageType::TransferInit,
            message_id: "mid-c".to_string(),
            transfer_id: "tr-c".to_string(),
            idempotency_key: "ik-c".to_string(),
            correlation_id: "corr".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: "2026-02-26T18:00:00Z".to_string(),
            expires_at: Some("2026-02-26T18:05:00Z".to_string()),
            payload: TransferInitPayload {
                asset: "lovelace".to_string(),
                amount: "10".to_string(),
                destination_account_ref: "acct".to_string(),
                source_intent_hash: "h".to_string(),
            },
            auth: auth.clone(),
        });

        let status = Request::CrossNodeTransferStatus(XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.7".to_string(),
            message_type: XNodeMessageType::TransferStatusQuery,
            message_id: "mid-s".to_string(),
            transfer_id: "tr-c".to_string(),
            idempotency_key: "ik-s".to_string(),
            correlation_id: "corr".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: "2026-02-26T18:00:00Z".to_string(),
            expires_at: None,
            payload: TransferStatusQueryPayload {
                query_type: TransferQueryType::Current,
            },
            auth: auth.clone(),
        });

        let ack = Request::CrossNodeTransferAck(XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.7".to_string(),
            message_type: XNodeMessageType::TransferAck,
            message_id: "mid-a".to_string(),
            transfer_id: "tr-c".to_string(),
            idempotency_key: "ik-a".to_string(),
            correlation_id: "corr".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: "2026-02-26T18:00:00Z".to_string(),
            expires_at: Some("2026-02-26T18:05:00Z".to_string()),
            payload: TransferAckPayload {
                ack_for_message_id: "mid-c".to_string(),
                ack_status: TransferAckStatus::Processed,
                ack_at: "2026-02-26T18:00:01Z".to_string(),
            },
            auth,
        });

        let c = serde_json::to_value(&create).unwrap();
        let s = serde_json::to_value(&status).unwrap();
        let a = serde_json::to_value(&ack).unwrap();

        assert_eq!(c["m"], "cross_node_transfer_create");
        assert_eq!(c["p"]["message_type"], "transfer_init");
        assert_eq!(s["m"], "cross_node_transfer_status");
        assert_eq!(s["p"]["payload"]["query_type"], "current");
        assert_eq!(a["m"], "cross_node_transfer_ack");
        assert_eq!(a["p"]["payload"]["ack_status"], "processed");
    }
}
