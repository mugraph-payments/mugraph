use std::collections::HashMap;

use mugraph_client::prelude::*;
use mugraph_core::programs::validate;
use proptest::{collection::hash_set, prelude::*};
use test_strategy::proptest;

// TODO: Make this much, much smarter.
//
// - It should generate 1..4 inputs, and 1..4 outputs
// - For the inputs and outputs, the asset ids must fully intersect
// - For each asset_id, the amounts in the inputs and outputs should equal.
// - Input notes should never have zero amounts
fn transaction() -> impl Strategy<Value = Transaction> {
    hash_set(any::<Note>(), 1..4).prop_map(|balances| {
        let mut builder = TransactionBuilder::new();

        for note in balances {
            builder = builder.input(&note).output(note.asset_id, note.amount);
        }

        builder.build()
    })
}

#[proptest]
fn test_transaction_strategy_is_balanced(#[strategy(transaction())] transaction: Transaction) {
    let mut input_balances = HashMap::new();
    let mut output_balances = HashMap::new();

    for i in 0..MAX_ATOMS {
        let asset_id_index = transaction.asset_id_indexes[i] as usize;
        let asset_id = transaction.asset_ids[asset_id_index];
        let amount = transaction.amounts[i];

        if transaction.input_mask.contains(i as u8) {
            input_balances
                .entry(asset_id)
                .and_modify(|x| *x += amount as u128)
                .or_insert(amount as u128);
        } else {
            output_balances
                .entry(asset_id)
                .and_modify(|x| *x += amount as u128)
                .or_insert(amount as u128);
        }
    }

    // Remove all zero balances
    input_balances.retain(|_, &mut v| v > 0);
    output_balances.retain(|_, &mut v| v > 0);

    prop_assert_eq!(input_balances, output_balances);
}

#[proptest]
fn test_validate_transaction(#[strategy(transaction())] transaction: Transaction) {
    validate(&transaction);
}

#[proptest]
#[should_panic]
fn test_validate_fails_if_unbalanced_amounts(
    #[strategy(transaction())] mut transaction: Transaction,
    #[strategy(1..u64::MAX)] amount: u64,
) {
    transaction.amounts[0] = transaction.amounts[0].saturating_add(amount);
    validate(&transaction)
}

#[proptest]
#[should_panic]
fn test_validate_fails_if_mismatching_asset_ids(
    #[strategy(transaction())] mut transaction: Transaction,
) {
    transaction.asset_id_indexes[0] += 1;

    validate(&transaction)
}
