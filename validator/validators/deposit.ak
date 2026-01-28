use aiken/collection/list
use aiken/crypto
use cardano/transaction.{OutputReference, Transaction}
use types.{DepositDatum, SpendRedeemer}

validator deposit {
  spend(
    datum: Option<DepositDatum>,
    redeemer: SpendRedeemer,
    _utxo: OutputReference,
    self: Transaction,
  ) {
    // Branch on datum presence; missing datum fails validation
    when datum is {
      Some(d) -> {
        // Ensure the provided node pubkey hashes to the stored hash
        trace @"Checking node pubkey hash matches datum"
        let node_pubkey_hash_matches =
          crypto.blake2b_224(redeemer.node_pubkey) == d.node_pubkey_hash

        // Verify node signature against transaction id using provided pubkey
        trace @"Verifying node signature"
        let node_valid =
          node_pubkey_hash_matches && crypto.verify_ed25519_signature(
            redeemer.node_pubkey,
            self.id,
            redeemer.node_signature,
          )
        // Verify user signature is in extra_signatories
        trace @"Verifying user signature"
        let user_signed =
          list.any(
            self.extra_signatories,
            fn(sig) { sig == d.user_pubkey_hash },
          )
        // Verify intent hash binding if present in datum
        trace @"Checking intent hash binding"
        let intent_valid =
          if d.intent_hash == #"" {
            // No intent hash required
            True
          } else {
            // Must match redeemer intent hash
            redeemer.intent_hash == d.intent_hash
          }
        // All conditions must be true
        trace @"Checking all validations pass"
        node_valid && user_signed && intent_valid
      }
      None -> {
        trace @"Missing datum"
        False
      }
    }
  }

  else(_) {
    fail
  }
}
