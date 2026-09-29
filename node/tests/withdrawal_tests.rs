//! Serde and encoding smoke tests for withdrawal request types.

use mugraph_core::types::{BlindedOutput, Note, WithdrawRequest};

#[test]
fn withdraw_request_serde_roundtrip_preserves_change_outputs() {
    let request = WithdrawRequest {
        notes: vec![Note::default()],
        change_outputs: vec![
            BlindedOutput::default(),
            BlindedOutput::default(),
        ],
        tx_cbor: "abcdef".to_string(),
        tx_hash: "hash123".to_string(),
    };

    let json = serde_json::to_string(&request).unwrap();
    let decoded: WithdrawRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.tx_hash, request.tx_hash);
    assert_eq!(decoded.tx_cbor, request.tx_cbor);
    assert_eq!(decoded.notes.len(), request.notes.len());
    assert_eq!(decoded.change_outputs.len(), request.change_outputs.len());
}
