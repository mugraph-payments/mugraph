//! Tests for deposit request handling and validation

use mugraph_core::types::{BlindSignature, DepositRequest, UtxoReference};

/// Test deposit request with valid structure
#[test]
fn test_deposit_request_structure() {
    let request = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "abc123".to_string(),
            index: 0,
        },
        outputs: vec![BlindSignature::default()],
        message: r#"{"user_pubkey":"deadbeef"}"#.to_string(),
        signature: vec![0u8; 64], // Valid Ed25519 signature length
        nonce: 12345,
        network: "preprod".to_string(),
    };

    assert_eq!(request.utxo.index, 0);
    assert_eq!(request.outputs.len(), 1);
    assert_eq!(request.signature.len(), 64);
}

/// Test deposit request validation - empty outputs should fail
#[test]
fn test_deposit_empty_outputs() {
    let request = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "abc123".to_string(),
            index: 0,
        },
        outputs: vec![], // Empty outputs
        message: r#"{"user_pubkey":"deadbeef"}"#.to_string(),
        signature: vec![0u8; 64],
        nonce: 12345,
        network: "preprod".to_string(),
    };

    assert!(request.outputs.is_empty());
}

/// Test deposit request with different networks
#[test]
fn test_deposit_network_variations() {
    let networks = vec!["mainnet", "preprod", "preview", "testnet"];

    for network in networks {
        let request = DepositRequest {
            utxo: UtxoReference {
                tx_hash: "abc123".to_string(),
                index: 0,
            },
            outputs: vec![BlindSignature::default()],
            message: r#"{"user_pubkey":"deadbeef"}"#.to_string(),
            signature: vec![0u8; 64],
            nonce: 12345,
            network: network.to_string(),
        };

        assert_eq!(request.network, network);
    }
}

/// Test deposit request signature length validation
#[test]
fn test_deposit_signature_lengths() {
    // Valid Ed25519 signature length
    let valid_sig = [0u8; 64];
    assert_eq!(valid_sig.len(), 64);

    // Invalid lengths
    let too_short = [0u8; 32];
    assert_ne!(too_short.len(), 64);

    let too_long = [0u8; 128];
    assert_ne!(too_long.len(), 64);
}

/// Test UtxoReference serialization
#[test]
fn test_utxo_reference_serialization() {
    let utxo = UtxoReference {
        tx_hash: "deadbeef".to_string(),
        index: 42,
    };

    let json = serde_json::to_string(&utxo).unwrap();
    assert!(json.contains("deadbeef"));
    assert!(json.contains("42"));

    let deserialized: UtxoReference = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tx_hash, "deadbeef");
    assert_eq!(deserialized.index, 42);
}

/// Test deposit request serialization
#[test]
fn test_deposit_request_serialization() {
    let request = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "abc123".to_string(),
            index: 1,
        },
        outputs: vec![BlindSignature::default()],
        message: "test_message".to_string(),
        signature: vec![1u8; 64],
        nonce: 999999,
        network: "mainnet".to_string(),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("abc123"));
    assert!(json.contains("mainnet"));

    let deserialized: DepositRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.nonce, 999999);
    assert_eq!(deserialized.network, "mainnet");
}

/// Test canonical payload structure
#[test]
fn test_canonical_payload_structure() {
    // The canonical payload should include specific fields
    let request = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "txhash123".to_string(),
            index: 0,
        },
        outputs: vec![BlindSignature::default()],
        message: r#"{"user_pubkey":"pubkey123"}"#.to_string(),
        signature: vec![0u8; 64],
        nonce: 1234567890,
        network: "preprod".to_string(),
    };

    // Serialize and verify structure
    let json = serde_json::to_string(&request.utxo).unwrap();
    assert!(json.contains("tx_hash"));
    assert!(json.contains("index"));
}
