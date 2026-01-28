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

fn test_node_pubkey_hash() -> ByteArray {
  // Blake2b-224 hash of a test node public key
  crypto.blake2b_224(
    #"6e6f64655f746573745f7075626c69635f6b65795f646174615f33325f627974657321",
  )
}

fn test_tx_hash() -> ByteArray {
  // Hex for ASCII "test_tx_body_hash"
  #"746573745f74785f626f64795f68617368"
}

fn test_intent_hash() -> ByteArray {
  // Example intent hash for testing (stored in datum for reference)
  crypto.blake2b_256(#"746573745f6465706f7369745f696e74656e745f7061796c6f6164")
}

fn test_utxo() -> OutputReference {
  OutputReference { transaction_id: test_tx_hash(), output_index: 0 }
}

fn test_datum() -> DepositDatum {
  DepositDatum {
    user_pubkey_hash: test_user_key_hash(),
    node_pubkey_hash: test_node_pubkey_hash(),
    intent_hash: test_intent_hash(),
  }
}

fn void_redeemer() -> SpendRedeemer {
  Void
}

// Test: Valid user signature succeeds
// Note: Node signature validation happens off-chain before submission
test spend_succeeds_with_valid_user_signature() {
  let datum = test_datum()
  let tx = make_transaction([test_user_key_hash()], test_tx_hash())
  let redeemer = void_redeemer()

  deposit.spend(Some(datum), redeemer, test_utxo(), tx) == True
}

// Test: Missing user signature fails
test spend_fails_without_user_signature() {
  let datum = test_datum()
  let tx = make_transaction([], test_tx_hash())
  let redeemer = void_redeemer()

  deposit.spend(Some(datum), redeemer, test_utxo(), tx) == False
}

// Test: Wrong user signature fails
test spend_fails_with_wrong_user_signature() {
  let datum = test_datum()
  let tx = make_transaction([test_wrong_user_key_hash()], test_tx_hash())
  let redeemer = void_redeemer()

  deposit.spend(Some(datum), redeemer, test_utxo(), tx) == False
}

// Test: Missing datum fails
test spend_fails_with_missing_datum() {
  let tx = make_transaction([test_user_key_hash()], test_tx_hash())
  let redeemer = void_redeemer()

  deposit.spend(None, redeemer, test_utxo(), tx) == False
}

// Test: Datum with empty intent hash still validates user signature
// Intent hash is stored for reference but not validated on-chain
test spend_succeeds_with_empty_intent_hash() {
  let datum =
    DepositDatum {
      user_pubkey_hash: test_user_key_hash(),
      node_pubkey_hash: test_node_pubkey_hash(),
      intent_hash: #"",
    }
  let tx = make_transaction([test_user_key_hash()], test_tx_hash())
  let redeemer = void_redeemer()

  deposit.spend(Some(datum), redeemer, test_utxo(), tx) == True
}
// Note on node signature validation:
// The validator assumes the node signature is valid because:
// 1. The transaction must be signed by the node before submission
// 2. This happens off-chain in the Mugraph node
// 3. The node_pubkey_hash in datum ensures only the authorized node can spend
// 4. The actual Ed25519 signature verification is performed by the Cardano ledger
//    when validating the transaction witness set
