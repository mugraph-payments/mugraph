use aiken/collection/dict
use aiken/crypto
use aiken/interval
use cardano/assets
use cardano/transaction.{OutputReference, Transaction}
use deposit
use types.{DepositDatum, SpendRedeemer}

// Test helper: Create a transaction with given extra_signatories and hash
fn make_transaction(
  extra_sigs: List<ByteArray>,
  tx_hash: ByteArray,
) -> Transaction {
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

fn test_user_key_hash() -> ByteArray {
  // Blake2b-224 hash of a test public key
  crypto.blake2b_224(
    #"757365725f746573745f7075626c69635f6b65795f646174615f33325f62797465735f21",
  )
}

fn test_wrong_user_key_hash() -> ByteArray {
  // Different key hash
  crypto.blake2b_224(
    #"77726f6e675f746573745f7075626c69635f6b65795f646174615f33325f6279746573",
  )
}

fn test_node_pubkey() -> ByteArray {
  // 32-byte Ed25519 public key used for node signature
  #"649d2b13e4b85553828c25a4cdf3f5e31f4230ba039029d03700e3481d6d79d3"
}

fn test_node_pubkey_hash() -> ByteArray {
  crypto.blake2b_224(test_node_pubkey())
}

fn wrong_node_pubkey() -> ByteArray {
  // Flip first byte to break the hash match
  #"659d2b13e4b85553828c25a4cdf3f5e31f4230ba039029d03700e3481d6d79d3"
}

fn test_node_signature() -> ByteArray {
  // Signature of "test_tx_body_hash" using the above node key
  #"eeae9489fc7472a297a2c7413822323f6a0b23786f7ac07eb6a15a9cb0439fae00580bbfb7b4a58da16abbe5ecb77cfcaffd556417e62fee528df22a337e8e0d"
}

fn invalid_node_signature() -> ByteArray {
  // Mutated signature to ensure verification fails
  #"efae9489fc7472a297a2c7413822323f6a0b23786f7ac07eb6a15a9cb0439fae00580bbfb7b4a58da16abbe5ecb77cfcaffd556417e62fee528df22a337e8e0d"
}

fn test_tx_hash() -> ByteArray {
  // Hex for ASCII "test_tx_body_hash"
  #"746573745f74785f626f64795f68617368"
}

fn test_intent_hash() -> ByteArray {
  // Example intent hash for testing
  crypto.blake2b_256(#"746573745f6465706f7369745f696e74656e745f7061796c6f6164")
}

fn test_utxo() -> OutputReference {
  OutputReference { transaction_id: test_tx_hash(), output_index: 0 }
}

fn datum_with_intent() -> DepositDatum {
  DepositDatum {
    user_pubkey_hash: test_user_key_hash(),
    node_pubkey_hash: test_node_pubkey_hash(),
    intent_hash: test_intent_hash(),
  }
}

fn datum_without_intent() -> DepositDatum {
  DepositDatum {
    user_pubkey_hash: test_user_key_hash(),
    node_pubkey_hash: test_node_pubkey_hash(),
    intent_hash: #"",
  }
}

fn valid_redeemer_with_intent() -> SpendRedeemer {
  SpendRedeemer {
    node_pubkey: test_node_pubkey(),
    node_signature: test_node_signature(),
    intent_hash: test_intent_hash(),
  }
}

// Test: Valid signatures and matching intent succeed
test spend_succeeds_with_node_and_user_signatures_and_intent() {
  let datum = datum_with_intent()
  let tx = make_transaction([test_user_key_hash()], test_tx_hash())
  let redeemer = valid_redeemer_with_intent()

  deposit.spend(Some(datum), redeemer, test_utxo(), tx) == True
}

// Test: Missing user signature fails
test spend_without_user_signature_fails() {
  let datum = datum_with_intent()
  let tx = make_transaction([], test_tx_hash())
  let redeemer = valid_redeemer_with_intent()

  deposit.spend(Some(datum), redeemer, test_utxo(), tx) == False
}

// Test: Wrong user signature fails
test spend_with_wrong_user_signature_fails() {
  let datum = datum_with_intent()
  let tx = make_transaction([test_wrong_user_key_hash()], test_tx_hash())
  let redeemer = valid_redeemer_with_intent()

  deposit.spend(Some(datum), redeemer, test_utxo(), tx) == False
}

// Test: Intent hash mismatch fails
test spend_with_intent_hash_mismatch_fails() {
  let datum = datum_with_intent()
  let tx = make_transaction([test_user_key_hash()], test_tx_hash())

  let redeemer =
    SpendRedeemer {
      node_pubkey: test_node_pubkey(),
      node_signature: test_node_signature(),
      intent_hash: crypto.blake2b_256(#"77726f6e675f696e74656e74"),
    }

  deposit.spend(Some(datum), redeemer, test_utxo(), tx) == False
}

// Test: Mismatched node pubkey hash fails
test spend_with_wrong_node_pubkey_fails() {
  let datum = datum_with_intent()
  let tx = make_transaction([test_user_key_hash()], test_tx_hash())

  let redeemer =
    SpendRedeemer {
      node_pubkey: wrong_node_pubkey(),
      node_signature: test_node_signature(),
      intent_hash: test_intent_hash(),
    }

  deposit.spend(Some(datum), redeemer, test_utxo(), tx) == False
}

// Test: Invalid node signature fails
test spend_with_invalid_node_signature_fails() {
  let datum = datum_with_intent()
  let tx = make_transaction([test_user_key_hash()], test_tx_hash())

  let redeemer =
    SpendRedeemer {
      node_pubkey: test_node_pubkey(),
      node_signature: invalid_node_signature(),
      intent_hash: test_intent_hash(),
    }

  deposit.spend(Some(datum), redeemer, test_utxo(), tx) == False
}

// Test: Empty intent hash in datum allows any redeemer intent
test spend_with_empty_intent_hash_allows_any() {
  let datum = datum_without_intent()
  let tx = make_transaction([test_user_key_hash()], test_tx_hash())

  let redeemer =
    SpendRedeemer {
      node_pubkey: test_node_pubkey(),
      node_signature: test_node_signature(),
      intent_hash: #"",
    }

  deposit.spend(Some(datum), redeemer, test_utxo(), tx) == True
}

// Test: Missing datum fails gracefully
test spend_with_missing_datum_fails() {
  let tx = make_transaction([test_user_key_hash()], test_tx_hash())
  let redeemer = valid_redeemer_with_intent()

  deposit.spend(None, redeemer, test_utxo(), tx) == False
}
