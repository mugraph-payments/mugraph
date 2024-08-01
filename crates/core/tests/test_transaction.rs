use mugraph_core::wallet::Wallet;
use proptest::prelude::*;
use test_strategy::proptest;

#[proptest]
fn test_send(alice: Wallet) {
    prop_assume!(!alice.notes.is_empty());
    todo!();
}
