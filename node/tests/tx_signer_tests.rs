//! Comprehensive tests for Cardano transaction signing and validation

use mugraph_node::tx_signer::{
    attach_witness_to_transaction,
    build_node_witness,
    compute_tx_hash,
    sign_transaction_body,
};
use whisky_csl::csl;

/// Build a minimal, valid transaction using cardano-serialization-lib
fn build_min_tx() -> csl::Transaction {
    let tx_hash = csl::TransactionHash::from_bytes(vec![0; 32]).unwrap();
    let input = csl::TransactionInput::new(&tx_hash, 0);
    let mut inputs = csl::TransactionInputs::new();
    inputs.add(&input);

    let addr =
        csl::Address::from_bech32("addr_test1vru4e2un2tq50q4rv6qzk7t8w34gjdtw3y2uzuqxzj0ldrqqactxh")
            .unwrap();
    let coin = csl::Coin::from_str("1000000").unwrap();
    let value = csl::Value::new(&coin);
    let output = csl::TransactionOutput::new(&addr, &value);
    let mut outputs = csl::TransactionOutputs::new();
    outputs.add(&output);

    let fee = csl::Coin::from_str("170000").unwrap();
    let body = csl::TransactionBody::new_tx_body(&inputs, &outputs, &fee);
    let witness_set = csl::TransactionWitnessSet::new();
    csl::Transaction::new(&body, &witness_set, None)
}

/// Test witness attachment to transaction with existing witnesses
#[test]
fn test_attach_witness_with_existing_witnesses() {
    let (sk, vk) = mugraph_node::cardano::generate_payment_keypair().unwrap();

    // Minimal transaction
    let mut tx = build_min_tx();
    let mut witness_set = tx.witness_set();
    let dummy_vkey = csl::Vkey::new(&csl::PublicKey::from_bytes(&[0u8; 32]).unwrap());
    let dummy_sig = csl::Ed25519Signature::from_bytes([0u8; 64].to_vec()).unwrap();
    let existing = csl::Vkeywitness::new(&dummy_vkey, &dummy_sig);
    let mut vkeys = csl::Vkeywitnesses::new();
    vkeys.add(&existing);
    witness_set.set_vkeys(&vkeys);
    tx = csl::Transaction::new(&tx.body(), &witness_set, tx.auxiliary_data());

    let tx_cbor = tx.to_bytes();
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
    let parsed = csl::Transaction::from_bytes(new_tx.clone()).unwrap();
    assert!(
        parsed
            .witness_set()
            .vkeys()
            .map(|v| v.len() >= 2)
            .unwrap_or(false)
    );
}

/// Test witness attachment preserves transaction structure
#[test]
fn test_witness_attachment_preserves_structure() {
    let (sk, vk) = mugraph_node::cardano::generate_payment_keypair().unwrap();

    // Create transaction with auxiliary data
    let mut tx = build_min_tx();
    tx.set_is_valid(true);
    let aux = csl::AuxiliaryData::new();
    tx = csl::Transaction::new(&tx.body(), &tx.witness_set(), Some(aux));

    let tx_cbor = tx.to_bytes();
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
    let parsed = csl::Transaction::from_bytes(new_tx).unwrap();
    assert!(parsed.is_valid());
    assert!(parsed.auxiliary_data().is_some());
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

    // Transaction with only body - should fail to parse
    let tx_cbor = vec![0x81, 0xa0]; // Array of 1, empty map
    assert!(compute_tx_hash(&tx_cbor).is_err());
    let result = attach_witness_to_transaction(&tx_cbor, &[0u8; 32], &wallet);
    assert!(result.is_err());
}

/// Test multiple witnesses in a single transaction
#[test]
fn test_multiple_witnesses() {
    let (sk1, vk1) = mugraph_node::cardano::generate_payment_keypair().unwrap();
    let (sk2, vk2) = mugraph_node::cardano::generate_payment_keypair().unwrap();

    let mut witnesses = csl::Vkeywitnesses::new();
    let sig1 = sign_transaction_body(&[1u8; 32], &sk1).unwrap();
    let sig2 = sign_transaction_body(&[2u8; 32], &sk2).unwrap();
    let v1 = csl::Vkeywitness::new(
        &csl::Vkey::new(&csl::PublicKey::from_bytes(&vk1).unwrap()),
        &csl::Ed25519Signature::from_bytes(sig1).unwrap(),
    );
    let v2 = csl::Vkeywitness::new(
        &csl::Vkey::new(&csl::PublicKey::from_bytes(&vk2).unwrap()),
        &csl::Ed25519Signature::from_bytes(sig2).unwrap(),
    );
    witnesses.add(&v1);
    witnesses.add(&v2);
    assert_eq!(witnesses.len(), 2);
}

/// Test compute_tx_hash with various transaction sizes
#[test]
fn test_compute_tx_hash_various_sizes() {
    let tx1 = build_min_tx();
    let hash1 = compute_tx_hash(&tx1.to_bytes()).unwrap();
    assert_eq!(hash1.len(), 32);

    // Same tx but different fee should change hash
    let mut body2 = tx1.body();
    body2.set_fee(&csl::Coin::from_str("200000").unwrap());
    let tx2 = csl::Transaction::new(&body2, &tx1.witness_set(), tx1.auxiliary_data());

    let hash2 = compute_tx_hash(&tx2.to_bytes()).unwrap();
    assert_eq!(hash2.len(), 32);
    assert_ne!(hash1, hash2);
}

/// Test that witness attachment creates valid CBOR
#[test]
fn test_witness_attachment_cbor_validity() {
    let (sk, vk) = mugraph_node::cardano::generate_payment_keypair().unwrap();

    let tx_cbor = build_min_tx().to_bytes();
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
    let parsed = csl::Transaction::from_bytes(new_tx).unwrap();
    let vkeys = parsed.witness_set().vkeys().unwrap();
    assert_eq!(vkeys.len(), 1);
    let vk_bytes = vkeys.get(0).vkey().public_key().as_bytes();
    assert_eq!(vk_bytes.len(), 32);
}

/// Test witness signature is valid
#[test]
fn test_witness_signature_valid() {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let (sk, vk) = mugraph_node::cardano::generate_payment_keypair().unwrap();

    let tx_cbor = build_min_tx().to_bytes();
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
    let parsed = csl::Transaction::from_bytes(new_tx).unwrap();
    let witness = parsed.witness_set().vkeys().unwrap().get(0);
    let stored_vk = witness.vkey().public_key().as_bytes();
    let stored_sig = witness.signature().to_bytes();

    // Verify the signature
    let verifying_key = VerifyingKey::from_bytes(&stored_vk).unwrap();
    let signature = Signature::from_slice(&stored_sig).unwrap();

    assert!(verifying_key.verify(&tx_hash, &signature).is_ok());
}
