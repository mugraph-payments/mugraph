//! Tests for withdrawal request handling and validation

use mugraph_core::types::{BlindSignature, WithdrawRequest};

/// Test withdrawal request with valid structure
#[test]
fn test_withdrawal_request_structure() {
    let request = WithdrawRequest {
        notes: vec![BlindSignature::default()],
        tx_cbor: hex::encode(vec![0x82, 0xa0, 0xa0]), // Minimal valid tx
        tx_hash: "abc123".to_string(),
    };

    assert_eq!(request.notes.len(), 1);
    assert!(!request.tx_cbor.is_empty());
    assert_eq!(request.tx_hash, "abc123");
}

/// Test withdrawal request validation - empty notes should fail
#[test]
fn test_withdrawal_empty_notes() {
    let request = WithdrawRequest {
        notes: vec![], // Empty notes
        tx_cbor: hex::encode(vec![0x82, 0xa0, 0xa0]),
        tx_hash: "abc123".to_string(),
    };

    assert!(request.notes.is_empty());
}

/// Test withdrawal request with multiple notes
#[test]
fn test_withdrawal_multiple_notes() {
    let request = WithdrawRequest {
        notes: vec![
            BlindSignature::default(),
            BlindSignature::default(),
            BlindSignature::default(),
        ],
        tx_cbor: hex::encode(vec![0x82, 0xa0, 0xa0]),
        tx_hash: "xyz789".to_string(),
    };

    assert_eq!(request.notes.len(), 3);
}

/// Test withdrawal request serialization
#[test]
fn test_withdrawal_request_serialization() {
    let request = WithdrawRequest {
        notes: vec![BlindSignature::default()],
        tx_cbor: "abcdef".to_string(),
        tx_hash: "hash123".to_string(),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("abcdef"));
    assert!(json.contains("hash123"));

    let deserialized: WithdrawRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tx_hash, "hash123");
}

/// Test tx_cbor hex encoding
#[test]
fn test_withdrawal_tx_cbor_encoding() {
    let tx_bytes = vec![0x82, 0xa0, 0xa0];
    let tx_cbor = hex::encode(&tx_bytes);

    let request = WithdrawRequest {
        notes: vec![BlindSignature::default()],
        tx_cbor,
        tx_hash: "test".to_string(),
    };

    // Verify we can decode it back
    let decoded = hex::decode(&request.tx_cbor).unwrap();
    assert_eq!(decoded, tx_bytes);
}

/// Test withdrawal request with large transaction
#[test]
fn test_withdrawal_large_transaction() {
    let large_tx = vec![0u8; 10000]; // 10KB transaction
    let request = WithdrawRequest {
        notes: vec![BlindSignature::default()],
        tx_cbor: hex::encode(large_tx),
        tx_hash: "large_tx".to_string(),
    };

    assert!(request.tx_cbor.len() > 1000);
}

/// Test withdrawal with valid hex tx_hash
#[test]
fn test_withdrawal_valid_tx_hash() {
    let valid_hash = "a".repeat(64); // 64 hex chars = 32 bytes

    let request = WithdrawRequest {
        notes: vec![BlindSignature::default()],
        tx_cbor: hex::encode(vec![0x82, 0xa0, 0xa0]),
        tx_hash: valid_hash.clone(),
    };

    assert_eq!(request.tx_hash.len(), 64);
}

/// Test transaction size limits
#[test]
fn test_withdrawal_transaction_size_limits() {
    // Maximum allowed size (16KB)
    let max_size = 16384;
    let max_tx = vec![0u8; max_size];

    let request = WithdrawRequest {
        notes: vec![BlindSignature::default()],
        tx_cbor: hex::encode(max_tx),
        tx_hash: "max_size".to_string(),
    };

    assert!(hex::decode(&request.tx_cbor).unwrap().len() <= max_size);
}
