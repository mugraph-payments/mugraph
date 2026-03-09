//! Serde smoke tests for deposit request types.

use mugraph_core::types::{BlindSignature, DepositRequest, UtxoReference};

#[test]
fn utxo_reference_serde_roundtrip_preserves_fields() {
    let utxo = UtxoReference {
        tx_hash: "deadbeef".to_string(),
        index: 42,
    };

    let json = serde_json::to_string(&utxo).unwrap();
    let decoded: UtxoReference = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.tx_hash, utxo.tx_hash);
    assert_eq!(decoded.index, utxo.index);
}

#[test]
fn deposit_request_wire_shape_still_serializes_with_message_field() {
    let request = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "abc123".to_string(),
            index: 1,
        },
        outputs: vec![BlindSignature::default()],
        message: "test_message".to_string(),
        signature: vec![1u8; 64],
        nonce: 999_999,
        network: "mainnet".to_string(),
    };

    let json = serde_json::to_value(&request).unwrap();
    let object = json.as_object().unwrap();

    assert!(object.contains_key("utxo"));
    assert!(object.contains_key("outputs"));
    assert!(object.contains_key("message"));
    assert!(object.contains_key("signature"));
    assert!(object.contains_key("nonce"));
    assert!(object.contains_key("network"));
    assert_eq!(object.get("message").unwrap(), "test_message");
}

#[test]
fn deposit_request_serde_roundtrip_preserves_nonce_and_network() {
    let request = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "abc123".to_string(),
            index: 1,
        },
        outputs: vec![BlindSignature::default()],
        message: "test_message".to_string(),
        signature: vec![1u8; 64],
        nonce: 999_999,
        network: "mainnet".to_string(),
    };

    let json = serde_json::to_string(&request).unwrap();
    let decoded: DepositRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.utxo.tx_hash, request.utxo.tx_hash);
    assert_eq!(decoded.utxo.index, request.utxo.index);
    assert_eq!(decoded.nonce, request.nonce);
    assert_eq!(decoded.network, request.network);
    assert_eq!(decoded.signature, request.signature);
}
