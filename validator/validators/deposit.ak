use cardano/transaction.{OutputReference, Transaction}
use deposit_logic
use types.{DepositDatum, SpendRedeemer}

validator deposit {
  spend(
    datum: Option<DepositDatum>,
    _redeemer: SpendRedeemer,
    _utxo: OutputReference,
    self: Transaction,
  ) {
    deposit_logic.validate_spend(datum, self)
  }

  else(_) {
    fail
  }
}
