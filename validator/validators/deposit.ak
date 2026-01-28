use aiken/builtin
use aiken/collection/list
use cardano/transaction.{OutputReference, Transaction}
use types.{DepositDatum, SpendRedeemer}

validator deposit {
  spend(
    datum: Option<DepositDatum>,
    _redeemer: SpendRedeemer,
    _utxo: OutputReference,
    self: Transaction,
  ) {
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

  else(_) {
    fail
  }
}
