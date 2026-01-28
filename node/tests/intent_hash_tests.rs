//! Tests for intent hash computation

use mugraph_core::types::{DepositRequest, PublicKey, UtxoReference};
use mugraph_node::routes::compute_intent_hash;

/// Test intent hash computation produces consistent results
#[test]
fn test_intent_hash_consistency() {
    let request = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "abc123".to_string(),
            index: 0,
        },
        outputs: vec![], // Empty for simplicity
        message: "test_message".to_string(),
        signature: vec![0u8; 64],
        nonce: 12345,
        network: "preprod".to_string(),
    };

    let delegate_pk = PublicKey::default();
    let script_address = "addr_test1...".to_string();

    // Compute hash twice with same inputs
    let hash1 = compute_intent_hash(&request, &delegate_pk, &script_address);
    let hash2 = compute_intent_hash(&request, &delegate_pk, &script_address);

    // Should be identical
    assert_eq!(hash1, hash2);
}

/// Test intent hash changes with different inputs
#[test]
fn test_intent_hash_uniqueness() {
    let request1 = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "abc123".to_string(),
            index: 0,
        },
        outputs: vec![],
        message: "test_message".to_string(),
        signature: vec![0u8; 64],
        nonce: 12345,
        network: "preprod".to_string(),
    };

    let request2 = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "def456".to_string(), // Different tx_hash
            index: 0,
        },
        outputs: vec![],
        message: "test_message".to_string(),
        signature: vec![0u8; 64],
        nonce: 12345,
        network: "preprod".to_string(),
    };

    let delegate_pk = PublicKey::default();
    let script_address = "addr_test1...".to_string();

    let hash1 = compute_intent_hash(&request1, &delegate_pk, &script_address);
    let hash2 = compute_intent_hash(&request2, &delegate_pk, &script_address);

    // Different inputs should produce different hashes
    assert_ne!(hash1, hash2);
}

/// Test intent hash with different nonces
#[test]
fn test_intent_hash_nonce_sensitivity() {
    let request1 = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "abc123".to_string(),
            index: 0,
        },
        outputs: vec![],
        message: "test_message".to_string(),
        signature: vec![0u8; 64],
        nonce: 12345,
        network: "preprod".to_string(),
    };

    let request2 = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "abc123".to_string(),
            index: 0,
        },
        outputs: vec![],
        message: "test_message".to_string(),
        signature: vec![0u8; 64],
        nonce: 12346, // Different nonce
        network: "preprod".to_string(),
    };

    let delegate_pk = PublicKey::default();
    let script_address = "addr_test1...".to_string();

    let hash1 = compute_intent_hash(&request1, &delegate_pk, &script_address);
    let hash2 = compute_intent_hash(&request2, &delegate_pk, &script_address);

    // Different nonce should produce different hash
    assert_ne!(hash1, hash2);
}

/// Test intent hash output length
#[test]
fn test_intent_hash_length() {
    let request = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "abc123".to_string(),
            index: 0,
        },
        outputs: vec![],
        message: "test_message".to_string(),
        signature: vec![0u8; 64],
        nonce: 12345,
        network: "preprod".to_string(),
    };

    let delegate_pk = PublicKey::default();
    let script_address = "addr_test1...".to_string();

    let hash = compute_intent_hash(&request, &delegate_pk, &script_address);

    // Should be 32 bytes (blake2b-256)
    assert_eq!(hash.len(), 32);
}

/// Test intent hash with different networks
#[test]
fn test_intent_hash_network_sensitivity() {
    let request1 = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "abc123".to_string(),
            index: 0,
        },
        outputs: vec![],
        message: "test_message".to_string(),
        signature: vec![0u8; 64],
        nonce: 12345,
        network: "preprod".to_string(),
    };

    let request2 = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "abc123".to_string(),
            index: 0,
        },
        outputs: vec![],
        message: "test_message".to_string(),
        signature: vec![0u8; 64],
        nonce: 12345,
        network: "mainnet".to_string(), // Different network
    };

    let delegate_pk = PublicKey::default();
    let script_address = "addr_test1...".to_string();

    let hash1 = compute_intent_hash(&request1, &delegate_pk, &script_address);
    let hash2 = compute_intent_hash(&request2, &delegate_pk, &script_address);

    // Different network should produce different hash
    assert_ne!(hash1, hash2);
}
