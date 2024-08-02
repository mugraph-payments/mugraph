use proptest::prelude::*;
use test_strategy::proptest;

#[proptest]
fn test_send() {
    prop_assert_eq!(1, 1);
}
