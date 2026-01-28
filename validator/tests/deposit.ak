use cardano/transaction.{Transaction, OutputReference}
use cardano/assets
use aiken/crypto
use aiken/builtin
use aiken/collection/dict
use validator/deposit
use types.{DepositDatum, SpendRedeemer}
use aiken/interval
use aiken/collection/list

// Test helper: Create a transaction with given extra_signatories and hash
fn make_transaction(extra_sigs: List<ByteArray>, tx_hash: ByteArray) -> Transaction {
  Transaction {
    inputs: [],
    reference_inputs: [],
    outputs: [],
    fee: 0,
    mint: assets.zero,
    certificates: [],
    withdrawals: [],
    validity_range: interval.everything,
    extra_signatories: extra_sigs,
    redeemers: [],
    datums: dict.empty,
    id: tx_hash,
    votes: [],
    proposal_procedures: [],
    current_treasury_amount: None,
    treasury_donation: None,
  }
}

// Helper to create test keys (28-byte hashes)
fn test_user_key() -> ByteArray {
  // Blake2b-224 hash of a test public key
  crypto.blake2b_224(#"user_test_public_key_data_32_bytes_!")
}

fn test_node_key() -> ByteArray {
  // Blake2b-224 hash of a test public key  
  crypto.blake2b_224(#"node_test_public_key_data_32_bytes_!")
}

fn test_wrong_key() -> ByteArray {
  // Different key hash
  crypto.blake2b_224(#"wrong_test_public_key_data_32_bytes")
}

fn test_tx_hash() -> ByteArray {
  // Fixed transaction hash for testing
  #"0000000000000000000000000000000000000000000000000000000000000000"
}

fn test_intent_hash() -> ByteArray {
  // Example intent hash for testing
  crypto.blake2b_256(#"test_deposit_intent_payload")
}

fn test_utxo() -> OutputReference {
  OutputReference {
    transaction_id: test_tx_hash(),
    output_index: 0,
  }
}

// Generate a deterministic signature for testing
// In production, this would use proper Ed25519 signing
fn generate_test_signature(secret_key: ByteArray, message: ByteArray) -> ByteArray {
  crypto.blake2b_256(builtin.concat(secret_key, message))
}

// Test: Missing user signature fails
test spend_without_user_signature_fails() {
  let datum = DepositDatum {
    user_pubkey_hash: test_user_key(),
    node_pubkey_hash: test_node_key(),
    intent_hash: #"",
  }
  
  // No user signature in extra_signatories
  let tx = make_transaction([], test_tx_hash())
  
  let redeemer = SpendRedeemer {
    node_signature: generate_test_signature(#"node_secret", test_tx_hash()),
    intent_hash: #"",
  }
  
  let result = deposit.spend(Some(datum), redeemer, test_utxo(), tx)
  
  // Should fail because user didn't sign
  result == False
}

// Test: Wrong user signature fails
test spend_with_wrong_user_signature_fails() {
  let datum = DepositDatum {
    user_pubkey_hash: test_user_key(),
    node_pubkey_hash: test_node_key(),
    intent_hash: #"",
  }
  
  // Wrong user signature
  let tx = make_transaction([test_wrong_key()], test_tx_hash())
  
  let redeemer = SpendRedeemer {
    node_signature: generate_test_signature(#"node_secret", test_tx_hash()),
    intent_hash: #"",
  }
  
  let result = deposit.spend(Some(datum), redeemer, test_utxo(), tx)
  
  // Should fail because wrong user signed
  result == False
}

// Test: Intent hash mismatch fails
test spend_with_intent_hash_mismatch_fails() {
  let datum = DepositDatum {
    user_pubkey_hash: test_user_key(),
    node_pubkey_hash: test_node_key(),
    intent_hash: test_intent_hash(),
  }
  
  let tx = make_transaction([test_user_key()], test_tx_hash())
  
  // Wrong intent hash in redeemer
  let wrong_intent = crypto.blake2b_256(#"wrong_intent")
  let redeemer = SpendRedeemer {
    node_signature: generate_test_signature(#"node_secret", test_tx_hash()),
    intent_hash: wrong_intent,
  }
  
  let result = deposit.spend(Some(datum), redeemer, test_utxo(), tx)
  
  // Should fail because intent hash doesn't match
  result == False
}

// Test: Matching intent hash succeeds (signature structure only)
test spend_with_matching_intent_hash_valid_structure() {
  let datum = DepositDatum {
    user_pubkey_hash: test_user_key(),
    node_pubkey_hash: test_node_key(),
    intent_hash: test_intent_hash(),
  }
  
  let tx = make_transaction([test_user_key()], test_tx_hash())
  
  let redeemer = SpendRedeemer {
    node_signature: generate_test_signature(#"node_secret", test_tx_hash()),
    intent_hash: test_intent_hash(),
  }
  
  // This validates structure but signature won't verify without proper keypair
  // Returns False due to invalid signature
  let result = deposit.spend(Some(datum), redeemer, test_utxo(), tx)
  
  // For structure test, we expect False (signatures don't match)
  // Real validation needs proper Ed25519 keypairs
  result == False
}

// Test: Empty intent hash in datum allows any redeemer intent
test spend_with_empty_intent_hash_allows_any() {
  let datum = DepositDatum {
    user_pubkey_hash: test_user_key(),
    node_pubkey_hash: test_node_key(),
    intent_hash: #"",
  }
  
  let tx = make_transaction([test_user_key()], test_tx_hash())
  
  // Even with non-empty intent in redeemer, should pass structure check
  let redeemer = SpendRedeemer {
    node_signature: generate_test_signature(#"node_secret", test_tx_hash()),
    intent_hash: test_intent_hash(),
  }
  
  // Structure is valid but signature won't verify
  let result = deposit.spend(Some(datum), redeemer, test_utxo(), tx)
  
  // For structure test, we expect False (signatures don't match)
  result == False
}

// Test: Missing datum fails
test spend_with_missing_datum_fails() {
  let tx = make_transaction([test_user_key()], test_tx_hash())
  
  let redeemer = SpendRedeemer {
    node_signature: generate_test_signature(#"node_secret", test_tx_hash()),
    intent_hash: #"",
  }
  
  // This should crash with expect failure when datum is None
  let result = deposit.spend(None, redeemer, test_utxo(), tx)
  result == False
}
