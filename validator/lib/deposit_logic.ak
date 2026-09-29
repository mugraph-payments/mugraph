use aiken/builtin
use aiken/collection/list
use cardano/transaction.{Transaction}
use types.{DepositDatum}

/// Core deposit validation logic, extracted for testability.
/// Checks:
/// 1. Datum is present
/// 2. user_pubkey_hash and node_pubkey_hash are each 28 bytes
/// 3. node_pubkey_hash appears in transaction extra_signatories
///
/// Only the node can spend a vault UTxO. The node holds the funds for the
/// holders of its notes, and it signs a withdrawal only after it burns
/// notes of the same value. If the depositor could also spend the UTxO,
/// they could spend their notes and then take the deposit back.
pub fn validate_spend(datum: Option<DepositDatum>, self: Transaction) -> Bool {
  when datum is {
    Some(d) -> {
      trace @"Checking datum key hash lengths"
      let valid_hash_lengths =
        builtin.length_of_bytearray(d.user_pubkey_hash) == 28 && builtin.length_of_bytearray(
          d.node_pubkey_hash,
        ) == 28

      trace @"Verifying node signature"
      let node_signed =
        list.any(
          self.extra_signatories,
          fn(sig) { sig == d.node_pubkey_hash },
        )

      trace @"Checking validations pass"
      valid_hash_lengths && node_signed
    }
    None -> {
      trace @"Missing datum"
      False
    }
  }
}
