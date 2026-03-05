//! Integration tests for Cardano deposit and withdrawal functionality

use mugraph_core::types::{DepositRequest, UtxoReference, WithdrawRequest};
use mugraph_node::{
    cardano::{build_script_address, compute_script_hash, generate_payment_keypair},
    provider::{Provider, UtxoInfo},
};

#[test]
fn test_keypair_generation() {
    let (sk, vk) = generate_payment_keypair().unwrap();
    assert_eq!(sk.len(), 32);
    assert_eq!(vk.len(), 32);
}

#[test]
fn test_script_hash_computation() {
    let cbor = vec![0x00, 0x01, 0x02, 0x03];
    let hash = compute_script_hash(&cbor);
    assert_eq!(hash.len(), 28); // Blake2b-224
}

#[test]
fn test_script_address_building() {
    let script_hash = vec![0u8; 28];

    // Test mainnet address
    let mainnet_addr = build_script_address(&script_hash, "mainnet").unwrap();
    assert!(mainnet_addr.starts_with("addr1"));

    // Test testnet address
    let testnet_addr = build_script_address(&script_hash, "preprod").unwrap();
    assert!(testnet_addr.starts_with("addr_test1"));
}

#[test]
fn test_provider_creation() {
    let provider = Provider::new(
        "blockfrost",
        "test_key".to_string(),
        "preprod".to_string(),
        None,
    );
    assert!(provider.is_ok());
}

#[test]
fn test_utxo_info_serialization() {
    let utxo = UtxoInfo {
        tx_hash: "abc123".to_string(),
        output_index: 0,
        address: "addr_test1...".to_string(),
        amount: vec![mugraph_node::provider::AssetAmount {
            unit: "lovelace".to_string(),
            quantity: "1000000".to_string(),
        }],
        datum_hash: None,
        datum: None,
        script_ref: None,
        block_height: Some(12345),
    };

    let json = serde_json::to_string(&utxo).unwrap();
    assert!(json.contains("abc123"));
    assert!(json.contains("lovelace"));
}

/// Test that deposit request payload round-trips with security-critical fields intact
#[test]
fn test_deposit_request_validation() {
    let request = DepositRequest {
        utxo: UtxoReference {
            tx_hash: "ab".repeat(32),
            index: 1,
        },
        outputs: vec![Default::default()],
        message: r#"{"user_pubkey":"11"}"#.to_string(),
        signature: vec![0u8; 64],
        nonce: 42,
        network: "preprod".to_string(),
    };

    let json = serde_json::to_string(&request).expect("serialize deposit request");
    let decoded: DepositRequest = serde_json::from_str(&json).expect("deserialize deposit request");

    assert_eq!(decoded.utxo.tx_hash, request.utxo.tx_hash);
    assert_eq!(decoded.utxo.index, request.utxo.index);
    assert_eq!(decoded.nonce, 42);
    assert_eq!(decoded.network, "preprod");
    assert_eq!(decoded.signature.len(), 64);
}

/// Test that withdrawal request carries CBOR hex that decodes deterministically
#[test]
fn test_withdrawal_request_validation() {
    let request = WithdrawRequest {
        notes: vec![Default::default()],
        tx_cbor: hex::encode([0x82u8, 0xA0, 0xA0]),
        tx_hash: "cd".repeat(32),
    };

    let decoded = hex::decode(&request.tx_cbor).expect("valid hex cbor");
    assert_eq!(decoded, vec![0x82, 0xA0, 0xA0]);

    let invalid = WithdrawRequest {
        tx_cbor: "zz-not-hex".to_string(),
        ..request
    };
    assert!(hex::decode(&invalid.tx_cbor).is_err());
}
