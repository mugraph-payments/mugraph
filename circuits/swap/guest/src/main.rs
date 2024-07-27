use std::collections::{BTreeMap, BTreeSet};

use mugraph_core::Transaction;
use risc0_zkvm::guest::env;
use risc0_zkvm::sha::{Impl, Sha256};

fn main() {
    let transaction: Transaction = env::read();
    let mut input_balances = BTreeMap::new();
    let mut output_balances = BTreeMap::new();
    let mut nullifiers = BTreeSet::new();

    for note in transaction.inputs {
        if let Some(note) = note {
            env::commit_slice(&note.nullifier);

            input_balances
                .entry(note.asset_id)
                .and_modify(|x| *x += note.amount)
                .or_insert(note.amount);

            assert!(!nullifiers.contains(&note.nullifier));

            nullifiers.insert(note.nullifier);
        }
    }

    for note in transaction.outputs {
        if let Some(note) = note {
            let bytes = [
                note.asset_id.as_ref(),
                note.amount.to_le_bytes().as_ref(),
                note.nullifier.as_ref(),
            ]
            .concat();

            env::commit_slice(&Impl::hash_bytes(&bytes).as_bytes());

            output_balances
                .entry(note.asset_id)
                .and_modify(|x| *x += note.amount)
                .or_insert(note.amount);

            assert!(!nullifiers.contains(&note.nullifier));

            nullifiers.insert(note.nullifier);
        }
    }

    assert_eq!(input_balances, output_balances);
}
