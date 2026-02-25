use aiken/crypto
use cardano/assets
use deposit_logic
use aiken/builtin
use aiken/fuzz
use mocktail.{
  complete, mock_pub_key_address, mock_tx_hash,
  mocktail_tx, required_signer_hash, tx_in, tx_out,
}
use types.{DepositDatum}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_user_key_hash() -> ByteArray {
  crypto.blake2b_224(
    #"757365725f746573745f7075626c69635f6b65795f646174615f33325f62797465735f21",
  )
}

fn test_wrong_user_key_hash() -> ByteArray {
  crypto.blake2b_224(
    #"77726f6e675f746573745f7075626c69635f6b65795f646174615f33325f6279746573",
  )
}

fn test_node_key_hash() -> ByteArray {
  crypto.blake2b_224(
    #"6e6f64655f746573745f7075626c69635f6b65795f646174615f33325f627974657321",
  )
}

fn test_intent_hash() -> ByteArray {
  crypto.blake2b_256(#"746573745f6465706f7369745f696e74656e745f7061796c6f6164")
}

fn test_datum() -> DepositDatum {
  DepositDatum {
    user_pubkey_hash: test_user_key_hash(),
    node_pubkey_hash: test_node_key_hash(),
    intent_hash: test_intent_hash(),
  }
}


// ---------------------------------------------------------------------------
// 1. spend_valid_user_signature
// ---------------------------------------------------------------------------
test spend_valid_user_signature() {
  let tx =
    mocktail_tx()
      |> required_signer_hash(True, test_user_key_hash())
      |> complete()

  deposit_logic.validate_spend(Some(test_datum()), tx) == True
}

// ---------------------------------------------------------------------------
// 2. spend_missing_user_signature
// ---------------------------------------------------------------------------
test spend_missing_user_signature() {
  let tx =
    mocktail_tx()
      |> complete()

  deposit_logic.validate_spend(Some(test_datum()), tx) == False
}

// ---------------------------------------------------------------------------
// 3. spend_wrong_user_signature
// ---------------------------------------------------------------------------
test spend_wrong_user_signature() {
  let tx =
    mocktail_tx()
      |> required_signer_hash(True, test_wrong_user_key_hash())
      |> complete()

  deposit_logic.validate_spend(Some(test_datum()), tx) == False
}

// ---------------------------------------------------------------------------
// 4. spend_missing_datum
// ---------------------------------------------------------------------------
test spend_missing_datum() {
  let tx =
    mocktail_tx()
      |> required_signer_hash(True, test_user_key_hash())
      |> complete()

  deposit_logic.validate_spend(None, tx) == False
}

// ---------------------------------------------------------------------------
// 5. spend_short_user_hash
// ---------------------------------------------------------------------------
test spend_short_user_hash() {
  let datum =
    DepositDatum {
      user_pubkey_hash: #"aabbccdd",
      node_pubkey_hash: test_node_key_hash(),
      intent_hash: test_intent_hash(),
    }
  let tx =
    mocktail_tx()
      |> required_signer_hash(True, #"aabbccdd")
      |> complete()

  deposit_logic.validate_spend(Some(datum), tx) == False
}

// ---------------------------------------------------------------------------
// 6. spend_short_node_hash
// ---------------------------------------------------------------------------
test spend_short_node_hash() {
  let datum =
    DepositDatum {
      user_pubkey_hash: test_user_key_hash(),
      node_pubkey_hash: #"deadbeef",
      intent_hash: test_intent_hash(),
    }
  let tx =
    mocktail_tx()
      |> required_signer_hash(True, test_user_key_hash())
      |> complete()

  deposit_logic.validate_spend(Some(datum), tx) == False
}

// ---------------------------------------------------------------------------
// 7. spend_long_user_hash
// ---------------------------------------------------------------------------
test spend_long_user_hash() {
  // 32 bytes instead of 28
  let long_hash =
    #"0102030405060708091011121314151617181920212223242526272829303132"
  let datum =
    DepositDatum {
      user_pubkey_hash: long_hash,
      node_pubkey_hash: test_node_key_hash(),
      intent_hash: test_intent_hash(),
    }
  let tx =
    mocktail_tx()
      |> required_signer_hash(True, long_hash)
      |> complete()

  deposit_logic.validate_spend(Some(datum), tx) == False
}

// ---------------------------------------------------------------------------
// 8. spend_with_multiple_signatories
// ---------------------------------------------------------------------------
test spend_with_multiple_signatories() {
  let tx =
    mocktail_tx()
      |> required_signer_hash(True, test_wrong_user_key_hash())
      |> required_signer_hash(True, test_user_key_hash())
      |> required_signer_hash(True, test_node_key_hash())
      |> complete()

  deposit_logic.validate_spend(Some(test_datum()), tx) == True
}

// ---------------------------------------------------------------------------
// 9. spend_with_inputs_and_outputs
// ---------------------------------------------------------------------------
test spend_with_inputs_and_outputs() {
  let tx =
    mocktail_tx()
      |> tx_in(
          True,
          mock_tx_hash(0),
          0,
          assets.from_lovelace(5_000_000),
          mock_pub_key_address(0, None),
        )
      |> tx_out(True, mock_pub_key_address(1, None), assets.from_lovelace(4_500_000))
      |> required_signer_hash(True, test_user_key_hash())
      |> complete()

  deposit_logic.validate_spend(Some(test_datum()), tx) == True
}

// ---------------------------------------------------------------------------
// 10. spend_empty_intent_hash
// ---------------------------------------------------------------------------
test spend_empty_intent_hash() {
  let datum =
    DepositDatum {
      user_pubkey_hash: test_user_key_hash(),
      node_pubkey_hash: test_node_key_hash(),
      intent_hash: #"",
    }
  let tx =
    mocktail_tx()
      |> required_signer_hash(True, test_user_key_hash())
      |> complete()

  deposit_logic.validate_spend(Some(datum), tx) == True
}

// ---------------------------------------------------------------------------
// Property-based tests
// ---------------------------------------------------------------------------

// Every iteration derives valid 28-byte hashes via blake2b_224, so the
// validator is always exercised (no skip-heavy predicates).

test spend_fuzz_valid_signer(seed via fuzz.bytearray()) {
  let user_key = crypto.blake2b_224(seed)
  let node_key =
    crypto.blake2b_224(builtin.append_bytearray(#"01", seed))
  let datum =
    DepositDatum {
      user_pubkey_hash: user_key,
      node_pubkey_hash: node_key,
      intent_hash: #"",
    }
  let tx =
    mocktail_tx()
      |> required_signer_hash(True, user_key)
      |> complete()

  deposit_logic.validate_spend(Some(datum), tx) == True
}

test spend_fuzz_wrong_signer(seed via fuzz.bytearray()) {
  let user_key = crypto.blake2b_224(seed)
  let wrong_key =
    crypto.blake2b_224(builtin.append_bytearray(#"ff", seed))
  let node_key =
    crypto.blake2b_224(builtin.append_bytearray(#"02", seed))
  if user_key != wrong_key {
    let datum =
      DepositDatum {
        user_pubkey_hash: user_key,
        node_pubkey_hash: node_key,
        intent_hash: #"",
      }
    let tx =
      mocktail_tx()
        |> required_signer_hash(True, wrong_key)
        |> complete()

    deposit_logic.validate_spend(Some(datum), tx) == False
  } else {
    // blake2b-224 collision -- astronomically unlikely, skip
    True
  }
}
