use risc0_zkvm::{guest::env, serde};

use crate::types::*;

#[inline(always)]
pub fn validate(transaction: &Transaction) {
    let mut balances = [0u64; 4];

    for i in 0..4 {
        let asset_id = transaction.inputs.asset_ids[i] as usize;
        let amount = transaction.inputs.amounts[i];

        assert!(asset_id < transaction.assets.len());
        assert_ne!(amount, 0);

        if transaction.inputs.program_id[i] != Hash::zero() {
            assert!(transaction.inputs.data[i] < 4);

            env::verify(
                transaction.inputs.program_id[i],
                &serde::to_vec(transaction).unwrap(),
            )
            .unwrap();
        }

        balances[asset_id] = balances[asset_id].checked_add(amount).unwrap();
    }

    for i in 0..4 {
        let asset_id = transaction.inputs.asset_ids[i] as usize;
        let amount = transaction.outputs.amounts[i];

        if transaction.outputs.program_id[i] != Hash::zero() {
            assert!(transaction.inputs.data[i] < 4);

            env::verify(
                transaction.outputs.program_id[i],
                &serde::to_vec(transaction).unwrap(),
            )
            .unwrap();
        }

        assert!(asset_id < transaction.assets.len(), "Invalid asset id");

        balances[asset_id] = balances[asset_id].checked_sub(amount).unwrap();
    }

    assert_eq!(balances, [0u64; 4]);
}
