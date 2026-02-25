use aiken/builtin
use aiken/collection/list
use cardano/transaction.{Transaction}
use types.{DepositDatum}

/// Core deposit validation logic, extracted for testability.
/// Checks:
/// 1. Datum is present
/// 2. user_pubkey_hash and node_pubkey_hash are each 28 bytes
/// 3. user_pubkey_hash appears in transaction extra_signatories
pub fn validate_spend(datum: Option<DepositDatum>, self: Transaction) -> Bool {
  when datum is {
    Some(d) -> {
      trace @"Checking datum key hash lengths"
      let valid_hash_lengths =
        builtin.length_of_bytearray(d.user_pubkey_hash) == 28 && builtin.length_of_bytearray(
          d.node_pubkey_hash,
        ) == 28

      trace @"Verifying user signature"
      let user_signed =
        list.any(
          self.extra_signatories,
          fn(sig) { sig == d.user_pubkey_hash },
        )

      trace @"Checking validations pass"
      valid_hash_lengths && user_signed
    }
    None -> {
      trace @"Missing datum"
      False
    }
  }
}
