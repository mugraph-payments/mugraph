use mugraph_core::types::*;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Transaction(Transaction),
    DoubleSpend(Transaction),
    RedeemFail(Transaction),
}
