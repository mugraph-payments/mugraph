//! Comprehensive tests for Cardano transaction signing and validation

use mugraph_node::tx_signer::{
    VKeyWitness,
    attach_witness_to_transaction,
    build_node_witness,
    compute_tx_hash,
    sign_transaction_body,
};

/// Test witness attachment to transaction with existing witnesses
#[test]
fn test_attach_witness_with_existing_witnesses() {
    let (sk, vk) = mugraph_node::cardano::generate_payment_keypair().unwrap();

    // Create a transaction with an existing witness
    // [empty_body, {0: [[existing_vkey, existing_sig]]}]
    let existing_witness = vec![
        0x82, // Array of 2 elements
        0x58, 0x20, // Bytes (32 bytes)
    ];

    // Build a minimal transaction with witness set containing one witness
    let mut tx_cbor = vec![0x82]; // Array of 2 elements
    tx_cbor.push(0xa0); // Empty map for body

    // Witness set with one existing witness
    tx_cbor.push(0xa1); // Map with 1 entry
    tx_cbor.push(0x00); // Key 0 (vkey witnesses)
    tx_cbor.push(0x81); // Array of 1 element
    tx_cbor.push(0x82); // Array of 2 elements (vkey, sig)
    tx_cbor.push(0x58);
    tx_cbor.push(32); // Bytes(32) for vkey
    tx_cbor.extend_from_slice(&[0u8; 32]); // 32 zero bytes for vkey
    tx_cbor.push(0x58);
    tx_cbor.push(64); // Bytes(64) for sig
    tx_cbor.extend_from_slice(&[0u8; 64]); // 64 zero bytes for sig

    let tx_hash = compute_tx_hash(&tx_cbor).unwrap();

    let wallet = mugraph_core::types::CardanoWallet::new(
        sk,
        vk,
        vec![],
        vec![],
        "addr_test...".to_string(),
        "preprod".to_string(),
    );

    let result = attach_witness_to_transaction(&tx_cbor, &tx_hash, &wallet);
    assert!(result.is_ok());

    let new_tx = result.unwrap();
    assert!(new_tx.len() > tx_cbor.len());

    // Verify we can decode the new transaction
    let mut decoder = minicbor::Decoder::new(&new_tx);
    let len = decoder.array().unwrap().unwrap();
    assert_eq!(len, 2);
}

/// Test witness attachment preserves transaction structure
#[test]
fn test_witness_attachment_preserves_structure() {
    let (sk, vk) = mugraph_node::cardano::generate_payment_keypair().unwrap();

    // Create transaction with auxiliary data
    let mut tx_cbor = vec![0x84]; // Array of 4 elements
    tx_cbor.push(0xa0); // Empty map for body
    tx_cbor.push(0xa0); // Empty map for witness set
    tx_cbor.push(0xf5); // true (is_valid)
    tx_cbor.push(0xa0); // Empty map for auxiliary data

    let tx_hash = compute_tx_hash(&tx_cbor).unwrap();

    let wallet = mugraph_core::types::CardanoWallet::new(
        sk,
        vk,
        vec![],
        vec![],
        "addr_test...".to_string(),
        "preprod".to_string(),
    );

    let result = attach_witness_to_transaction(&tx_cbor, &tx_hash, &wallet);
    assert!(result.is_ok());

    let new_tx = result.unwrap();

    // Verify structure is preserved
    let mut decoder = minicbor::Decoder::new(&new_tx);
    let len = decoder.array().unwrap().unwrap();
    assert_eq!(len, 4);

    // Verify is_valid and auxiliary data are preserved
    decoder.skip().unwrap(); // skip body
    decoder.skip().unwrap(); // skip witness set
    let is_valid: bool = decoder.bool().unwrap();
    assert!(is_valid);
}

/// Test that invalid transaction CBOR is rejected
#[test]
fn test_attach_witness_invalid_cbor() {
    let (sk, vk) = mugraph_node::cardano::generate_payment_keypair().unwrap();
    let wallet = mugraph_core::types::CardanoWallet::new(
        sk,
        vk,
        vec![],
        vec![],
        "addr_test...".to_string(),
        "preprod".to_string(),
    );

    // Invalid CBOR - just raw bytes, not an array
    let invalid_tx = vec![0x00, 0x01, 0x02];
    let tx_hash = [0u8; 32];

    let result = attach_witness_to_transaction(&invalid_tx, &tx_hash, &wallet);
    assert!(result.is_err());
}

/// Test transaction with only body (no witness set)
#[test]
fn test_attach_witness_single_element() {
    let (sk, vk) = mugraph_node::cardano::generate_payment_keypair().unwrap();
    let wallet = mugraph_core::types::CardanoWallet::new(
        sk,
        vk,
        vec![],
        vec![],
        "addr_test...".to_string(),
        "preprod".to_string(),
    );

    // Transaction with only body - should fail
    let tx_cbor = vec![0x81, 0xa0]; // Array of 1, empty map
    let tx_hash = compute_tx_hash(&tx_cbor).unwrap();

    let result = attach_witness_to_transaction(&tx_cbor, &tx_hash, &wallet);
    assert!(result.is_err());
}

/// Test multiple witnesses in a single transaction
#[test]
fn test_multiple_witnesses() {
    let (sk1, vk1) = mugraph_node::cardano::generate_payment_keypair().unwrap();
    let (sk2, vk2) = mugraph_node::cardano::generate_payment_keypair().unwrap();

    // Create witness set with multiple witnesses manually
    let mut witness_set = vec![0xa1, 0x00, 0x82]; // Map, key 0, array of 2

    // First witness
    witness_set.push(0x82); // Array of 2
    witness_set.push(0x58);
    witness_set.push(32);
    witness_set.extend_from_slice(&vk1);
    witness_set.push(0x58);
    witness_set.push(64);
    // Create a dummy signature
    let sig1 = sign_transaction_body(&[1u8; 32], &sk1).unwrap();
    witness_set.extend_from_slice(&sig1);

    // Second witness
    witness_set.push(0x82); // Array of 2
    witness_set.push(0x58);
    witness_set.push(32);
    witness_set.extend_from_slice(&vk2);
    witness_set.push(0x58);
    witness_set.push(64);
    let sig2 = sign_transaction_body(&[2u8; 32], &sk2).unwrap();
    witness_set.extend_from_slice(&sig2);

    // Verify we can parse multiple witnesses
    let mut decoder = minicbor::Decoder::new(&witness_set);
    decoder.map().unwrap(); // Skip map header
    decoder.u64().unwrap(); // Skip key
    let arr_len = decoder.array().unwrap().unwrap();
    assert_eq!(arr_len, 2);
}

/// Test compute_tx_hash with various transaction sizes
#[test]
fn test_compute_tx_hash_various_sizes() {
    // Empty body
    let tx1 = vec![0x82, 0xa0, 0xa0];
    let hash1 = compute_tx_hash(&tx1).unwrap();
    assert_eq!(hash1.len(), 32);

    // Body with content
    let mut tx2 = vec![0x82]; // Array of 2
    tx2.push(0xa1); // Map with 1 entry
    tx2.push(0x02); // Key 2 (fee)
    minicbor::encode(1000000u64, &mut tx2).unwrap();
    tx2.push(0xa0); // Empty witness set

    let hash2 = compute_tx_hash(&tx2).unwrap();
    assert_eq!(hash2.len(), 32);
    assert_ne!(hash1, hash2); // Different transactions should have different hashes
}

/// Test that witness attachment creates valid CBOR
#[test]
fn test_witness_attachment_cbor_validity() {
    let (sk, vk) = mugraph_node::cardano::generate_payment_keypair().unwrap();

    let tx_cbor = vec![0x82, 0xa0, 0xa0];
    let tx_hash = compute_tx_hash(&tx_cbor).unwrap();

    let wallet = mugraph_core::types::CardanoWallet::new(
        sk,
        vk,
        vec![],
        vec![],
        "addr_test...".to_string(),
        "preprod".to_string(),
    );

    let new_tx = attach_witness_to_transaction(&tx_cbor, &tx_hash, &wallet).unwrap();

    // Verify the result is valid CBOR by parsing it completely
    let mut decoder = minicbor::Decoder::new(&new_tx);
    let len = decoder.array().unwrap().unwrap();
    assert_eq!(len, 2);

    // Parse body
    decoder.skip().unwrap();

    // Parse witness set
    let witness_map_len = decoder.map().unwrap().unwrap();
    assert!(witness_map_len >= 1);

    // Parse key and value
    let key: u64 = decoder.u64().unwrap();
    assert_eq!(key, 0);

    let arr_len = decoder.array().unwrap().unwrap();
    assert_eq!(arr_len, 1); // One witness added

    // Parse witness pair
    let pair_len = decoder.array().unwrap().unwrap();
    assert_eq!(pair_len, 2);

    let vkey: &[u8] = decoder.bytes().unwrap();
    assert_eq!(vkey.len(), 32);

    let sig: &[u8] = decoder.bytes().unwrap();
    assert_eq!(sig.len(), 64);
}

/// Test witness signature is valid
#[test]
fn test_witness_signature_valid() {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let (sk, vk) = mugraph_node::cardano::generate_payment_keypair().unwrap();

    let tx_cbor = vec![0x82, 0xa0, 0xa0];
    let tx_hash = compute_tx_hash(&tx_cbor).unwrap();

    let wallet = mugraph_core::types::CardanoWallet::new(
        sk.clone(),
        vk.clone(),
        vec![],
        vec![],
        "addr_test...".to_string(),
        "preprod".to_string(),
    );

    let new_tx = attach_witness_to_transaction(&tx_cbor, &tx_hash, &wallet).unwrap();

    // Extract the witness from the transaction
    let mut decoder = minicbor::Decoder::new(&new_tx);
    decoder.array().unwrap(); // Skip array header
    decoder.skip().unwrap(); // Skip body

    decoder.map().unwrap(); // Skip map header
    decoder.u64().unwrap(); // Skip key
    decoder.array().unwrap(); // Skip array header
    decoder.array().unwrap(); // Skip pair header

    let stored_vk: &[u8] = decoder.bytes().unwrap();
    let stored_sig: &[u8] = decoder.bytes().unwrap();

    // Verify the signature
    let verifying_key = VerifyingKey::from_bytes(stored_vk.try_into().unwrap()).unwrap();
    let signature = Signature::from_slice(stored_sig).unwrap();

    assert!(verifying_key.verify(&tx_hash, &signature).is_ok());
}
