use alloc::collections::BTreeMap;

use crate::types::*;

#[inline(always)]
#[no_mangle]
pub fn validate(transaction: Transaction) {
    let mut balances = BTreeMap::new();

    for i in 0..MAX_ATOMS {
        let asset_id_index = transaction.blob.asset_id_indexes[i] as usize;
        let asset_id = transaction.blob.asset_ids[asset_id_index];
        let amount = transaction.blob.amounts[i];

        match balances.get(&asset_id) {
            Some(b) => {
                if transaction.blob.parent_ids[i] == Hash::default() {
                    balances.insert(asset_id, b - amount as u128);
                } else {
                    balances.insert(asset_id, b + amount as u128);
                }
            }
            None => {
                balances.insert(asset_id, amount as u128);
            }
        }
    }

    for (_, balance) in balances.iter() {
        assert_eq!(*balance, 0);
    }
}

#[cfg(all(test, feature = "proptest"))]
mod tests {
    use std::collections::HashMap;

    use ::proptest::{collection::hash_set, prelude::*};
    use test_strategy::proptest;

    use super::*;

    // TODO: Make this much, much smarter.
    //
    // - It should generate 1..4 inputs, and 1..4 outputs
    // - For the inputs and outputs, the asset ids must fully intersect
    // - For each asset_id, the amounts in the inputs and outputs should equal.
    // - Notes could/should have programs, right now they are not implemented.
    // - Input notes should never have zero amounts
    fn transaction() -> impl Strategy<Value = Transaction> {
        let balances = hash_set(any::<Note>(), 1..4);

        (balances, any::<Manifest>()).prop_map(|(balances, manifest)| {
            let mut builder = TransactionBuilder::new(manifest);

            for mut note in balances {
                note.program_id = None;
                note.datum = None;

                builder = builder
                    .input(&note)
                    .output(note.asset_id, note.amount, None, None);
            }

            builder.build()
        })
    }

    #[proptest]
    fn test_transaction_strategy_is_balanced(#[strategy(transaction())] transaction: Transaction) {
        let mut input_balances = HashMap::new();
        let mut output_balances = HashMap::new();

        for i in 0..MAX_ATOMS {
            let asset_id_index = transaction.blob.asset_id_indexes[i] as usize;
            let asset_id = transaction.blob.asset_ids[asset_id_index];
            let amount = transaction.blob.amounts[i];

            if transaction.blob.parent_ids[i] == Hash::default() {
                output_balances
                    .entry(asset_id)
                    .and_modify(|x| *x += amount as u128)
                    .or_insert(amount as u128);
            } else {
                input_balances
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
        validate(transaction);
    }

    #[proptest]
    #[should_panic]
    fn test_validate_fails_if_unbalanced_amounts(
        #[strategy(transaction())] mut transaction: Transaction,
        #[strategy(1..u64::MAX)] amount: u64,
    ) {
        transaction.blob.amounts[0] = transaction.blob.amounts[0].saturating_add(amount);
        validate(transaction)
    }

    #[proptest]
    #[should_panic]
    fn test_validate_fails_if_mismatching_asset_ids(
        #[strategy(transaction())] mut transaction: Transaction,
    ) {
        transaction.blob.asset_id_indexes[0] += 1;

        validate(transaction)
    }
}
