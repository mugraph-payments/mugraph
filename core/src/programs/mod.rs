use crate::types::*;

#[inline(always)]
pub fn validate(transaction: &Transaction) {
    let mut balances = [0u64; 4];

    for i in 0..4 {
        let asset_id = transaction.inputs.asset_ids[i] as usize;
        let amount = transaction.inputs.amounts[i];

        assert!(asset_id < transaction.assets.len());
        assert_ne!(amount, 0);

        balances[asset_id] = balances[asset_id].checked_add(amount).unwrap();
    }

    for i in 0..4 {
        let asset_id = transaction.inputs.asset_ids[i] as usize;
        let amount = transaction.outputs.amounts[i];

        assert!(asset_id < transaction.assets.len(), "Invalid asset id");

        balances[asset_id] = balances[asset_id].checked_sub(amount).unwrap();
    }

    assert_eq!(balances, [0u64; 4]);
}
