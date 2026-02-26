use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::types::*;

#[derive(Debug, Clone, Serialize, Deserialize, Arbitrary)]
#[serde(tag = "m", content = "r")]
pub enum Response {
    #[serde(rename = "refresh")]
    Transaction {
        #[serde(rename = "s")]
        outputs: Vec<BlindSignature>,
    },
    #[serde(rename = "public_key")]
    Info {
        /// Node delegate public key
        delegate_pk: PublicKey,
        /// Cardano script address for deposits
        cardano_script_address: Option<String>,
    },
    #[serde(rename = "emit")]
    Emit(Box<Note>),
    #[serde(rename = "deposit")]
    Deposit {
        /// Blind signatures for the outputs
        #[serde(rename = "s")]
        signatures: Vec<BlindSignature>,
        /// Deposit reference (UTxO identifier: tx_hash:index)
        deposit_ref: String,
    },
    #[serde(rename = "withdraw")]
    Withdraw {
        /// Fully signed transaction CBOR (hex encoded)
        signed_tx_cbor: String,
        /// Transaction hash
        tx_hash: String,
        /// Change notes (if any)
        #[serde(rename = "s")]
        change_notes: Vec<BlindSignature>,
    },
    #[serde(rename = "cross_node_transfer_create")]
    CrossNodeTransferCreate {
        transfer_id: String,
        accepted: bool,
    },
    #[serde(rename = "cross_node_transfer_notify")]
    CrossNodeTransferNotify {
        accepted: bool,
    },
    #[serde(rename = "cross_node_transfer_status")]
    CrossNodeTransferStatus(Box<XNodeEnvelope<TransferStatusPayload>>),
    #[serde(rename = "cross_node_transfer_ack")]
    CrossNodeTransferAck {
        accepted: bool,
    },
    #[serde(rename = "error")]
    Error { reason: String },
}

#[cfg(test)]
mod tests {
    use proptest::prop_assert_eq;
    use test_strategy::proptest;

    use crate::types::{
        Response, TransferChainState, TransferCreditState, TransferSettlementState,
        TransferStatusPayload, XNodeAuth, XNodeEnvelope, XNodeMessageType,
    };

    #[proptest]
    fn test_serde_roundtrip(response: Response) {
        let json = serde_json::to_string(&response).unwrap();
        let decoded: Response = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&decoded).unwrap();
        prop_assert_eq!(json, json2);
    }

    #[test]
    fn test_cross_node_transfer_status_serialization() {
        let response = Response::CrossNodeTransferStatus(Box::new(XNodeEnvelope {
            m: "xnode".to_string(),
            version: "3.0".to_string(),
            message_type: XNodeMessageType::TransferStatus,
            message_id: "mid-1".to_string(),
            transfer_id: "tr-1".to_string(),
            idempotency_key: "ik-1".to_string(),
            correlation_id: "corr-1".to_string(),
            origin_node_id: "node://a".to_string(),
            destination_node_id: "node://b".to_string(),
            sent_at: "2026-02-26T18:00:00Z".to_string(),
            expires_at: None,
            payload: TransferStatusPayload {
                source_state: "confirming".to_string(),
                destination_state: "credit_eligible".to_string(),
                settlement_state: TransferSettlementState::Confirming,
                chain_state: TransferChainState::Confirming,
                credit_state: TransferCreditState::Eligible,
                tx_hash: Some("abcd".to_string()),
                confirmations_observed: 5,
                updated_at: "2026-02-26T18:00:00Z".to_string(),
            },
            auth: XNodeAuth {
                alg: "Ed25519".to_string(),
                kid: "k1".to_string(),
                sig: "sig".to_string(),
            },
        }));

        let value = serde_json::to_value(&response).unwrap();
        assert_eq!(value["m"], "cross_node_transfer_status");
        assert_eq!(value["r"]["payload"]["chain_state"], "confirming");
        assert_eq!(value["r"]["payload"]["credit_state"], "eligible");
    }

    #[test]
    fn test_cross_node_response_contract_shapes() {
        let create = Response::CrossNodeTransferCreate {
            transfer_id: "tr-1".to_string(),
            accepted: true,
        };
        let notify = Response::CrossNodeTransferNotify { accepted: true };
        let ack = Response::CrossNodeTransferAck { accepted: true };

        let c = serde_json::to_value(&create).unwrap();
        let n = serde_json::to_value(&notify).unwrap();
        let a = serde_json::to_value(&ack).unwrap();

        assert_eq!(c["m"], "cross_node_transfer_create");
        assert_eq!(c["r"]["accepted"], true);
        assert_eq!(n["m"], "cross_node_transfer_notify");
        assert_eq!(n["r"]["accepted"], true);
        assert_eq!(a["m"], "cross_node_transfer_ack");
        assert_eq!(a["r"]["accepted"], true);
    }
}
