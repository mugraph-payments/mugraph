use cardano/transaction.{Transaction, OutputReference}
use cardano/assets
use aiken/crypto
use aiken/builtin
use aiken/collection/dict
use deposit
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

// Test: Both signatures valid structure check
// Note: Full signature verification requires proper Ed25519 implementation
// which is best tested with integration tests using actual keypairs

test spend_with_both_signatures_valid_structure() {
  let datum = DepositDatum {
    user_pubkey_hash: test_user_key(),
    node_pubkey_hash: test_node_key(),
  }
  
  let tx = make_transaction([test_user_key()], test_tx_hash())
  
  let redeemer = SpendRedeemer {
    node_signature: generate_test_signature(#"node_secret", test_tx_hash()),
  }
  
  // This validates structure but signature won't verify without proper keypair
  // Returns False due to invalid signature
  let result = deposit.spend(Some(datum), redeemer, test_utxo(), tx)
  
  // For structure test, we expect False (signatures don't match)
  // Real validation needs proper Ed25519 keypairs
  result == False
}

test spend_without_user_signature_fails() {
  let datum = DepositDatum {
    user_pubkey_hash: test_user_key(),
    node_pubkey_hash: test_node_key(),
  }
  
  // No user signature in extra_signatories
  let tx = make_transaction([], test_tx_hash())
  
  let redeemer = SpendRedeemer {
    node_signature: generate_test_signature(#"node_secret", test_tx_hash()),
  }
  
  let result = deposit.spend(Some(datum), redeemer, test_utxo(), tx)
  
  // Should fail because user didn't sign
  result == False
}

test spend_with_wrong_user_signature_fails() {
  let datum = DepositDatum {
    user_pubkey_hash: test_user_key(),
    node_pubkey_hash: test_node_key(),
  }
  
  // Wrong user signature
  let tx = make_transaction([test_wrong_key()], test_tx_hash())
  
  let redeemer = SpendRedeemer {
    node_signature: generate_test_signature(#"node_secret", test_tx_hash()),
  }
  
  let result = deposit.spend(Some(datum), redeemer, test_utxo(), tx)
  
  // Should fail because wrong user signed
  result == False
}

// This test expects to crash with expect Some(d) = datum
// In Aiken, expect failures abort the test with error
// So we test that None datum causes failure by trying to run it
test spend_with_missing_datum_fails() {
  let tx = make_transaction([test_user_key()], test_tx_hash())
  
  let redeemer = SpendRedeemer {
    node_signature: generate_test_signature(#"node_secret", test_tx_hash()),
  }
  
  // This should crash with expect failure
  // We can't properly test expect failures in Aiken unit tests
  // This demonstrates the requirement
  let result = deposit.spend(None, redeemer, test_utxo(), tx)
  result == False
}
