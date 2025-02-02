use std::cmp::min;

use prop::collection::vec;
use proptest::prelude::*;
use rand::{prelude::*, rngs::OsRng};

use crate::protocol::*;

pub(crate) fn distribute_numbers(
    amount: u64,
    output_count: usize,
) -> impl Strategy<Value = Vec<u64>> {
    assert!(
        output_count < u8::MAX as usize,
        "Output count should never be too big."
    );

    vec(1..=amount / output_count as u64, output_count - 1).prop_map(move |mut v| {
        let sum: u64 = v.iter().sum();
        v.push(amount.saturating_sub(sum));

        while v.iter().any(|&x| x == 0) {
            let zeros = v.iter().filter(|&&x| x == 0).count();
            let excess: u64 = v.iter().filter(|&&x| x > 1).map(|&x| x - 1).sum();
            let distribution = excess.min(zeros as u64);

            for x in v.iter_mut() {
                if *x == 0 && distribution > 0 {
                    *x += 1;
                } else if *x > 1 && distribution > 0 {
                    *x -= 1;
                }
            }
        }

        v
    })
}

pub(crate) fn distribute(
    inputs: usize,
    outputs: usize,
) -> impl Strategy<Value = (Vec<Note>, Vec<Note>)> {
    assert_ne!(inputs, 0);
    assert_ne!(outputs, 0);
    assert!(
        inputs < u8::MAX as usize,
        "Input count should never be too big"
    );
    assert!(
        outputs < u8::MAX as usize,
        "Output count should never be too big"
    );

    let i = inputs;
    let input_notes = vec(
        any::<Note>().prop_map(move |mut n| {
            n.amount %= u64::MAX / i as u64;
            n
        }),
        min(inputs, outputs),
    );
    let output_nonces = vec(any::<Hash>(), outputs);

    (input_notes, output_nonces).prop_flat_map(move |(notes, nonces)| {
        let total_amount: u64 = notes.iter().map(|n| n.amount).sum();
        let output_amounts = distribute_numbers(total_amount, outputs);

        (output_amounts).prop_map(move |amounts| {
            let mut output_notes = Vec::new();
            let mut nonces = nonces.clone();

            for amount in amounts {
                if amount > 0 {
                    let input = notes.choose(&mut OsRng).unwrap();

                    output_notes.push(Note {
                        amount,
                        asset_id: input.asset_id,
                        asset_name: input.asset_name,
                        nonce: nonces.pop().unwrap(),
                    });
                }
            }

            (notes.clone(), output_notes)
        })
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn test_distribute_numbers(
        #[strategy(1u64..=u64::MAX / 2)] amount: u64,
        #[strategy(((#amount / u64::MAX) + 1..u8::MAX as u64).prop_map(|o| o as usize))]
        output_count: usize,
        #[strategy(distribute_numbers(#amount, #output_count))] numbers: Vec<u64>,
    ) {
        prop_assert_eq!(numbers.len(), output_count);
        prop_assert_eq!(numbers.into_iter().sum::<u64>(), amount);
    }

    #[proptest]
    fn test_distributed_notes_always_balance(
        #[strategy(1usize..64)] _i: usize,
        #[strategy(1usize..=#_i)] _o: usize,
        #[strategy(distribute(#_i, #_o))] notes: (Vec<Note>, Vec<Note>),
    ) {
        let mut pre = HashMap::new();
        let mut post = HashMap::new();
        let (inputs, outputs) = notes;

        for input in inputs {
            *pre.entry((input.asset_id, input.asset_name))
                .or_insert(0u128) += input.amount as u128;
        }

        for output in outputs {
            *post
                .entry((output.asset_id, output.asset_name))
                .or_insert(0u128) += output.amount as u128;
        }

        prop_assert_eq!(pre.values().sum::<u128>(), post.values().sum::<u128>());
    }
}
