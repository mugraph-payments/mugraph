use aiken/collection/list
use aiken/crypto
use cardano/transaction.{OutputReference, Transaction}
use types.{DepositDatum, SpendRedeemer}

pub fn find_matching_witness(
  witnesses: List<(ByteArray, ByteArray)>,
  expected_hash: ByteArray,
) -> Option<(ByteArray, ByteArray)> {
  list.find(
    witnesses,
    fn(witness) {
      let (pubkey, _signature) = witness
      crypto.blake2b_224(pubkey) == expected_hash
    },
  )
}

validator deposit {
  spend(
    datum: Option<DepositDatum>,
    _redeemer: SpendRedeemer,
    _utxo: OutputReference,
    self: Transaction,
  ) {
    when datum is {
      Some(d) -> {
        trace @"Verifying user signature"
        let user_signed =
          list.any(
            self.extra_signatories,
            fn(sig) { sig == d.user_pubkey_hash },
          )

        trace @"Checking validations pass"
        user_signed
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
