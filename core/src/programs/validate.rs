use crate::types::*;

#[inline(always)]
#[no_mangle]
pub fn validate(transaction: &Transaction) {
    let balances = [0u64; 4];

    // Process inputs
    for i in 0..4 {
        let asset_id = transaction.inputs.asset_ids[i] as usize;
        assert!(asset_id < transaction.assets.len(), "Invalid asset id");

        balances[asset_id]
            .checked_add(transaction.inputs.amounts[i])
            .unwrap();
    }

    // Process outputs
    for i in 0..4 {
        let asset_id = transaction.inputs.asset_ids[i] as usize;
        assert!(asset_id < transaction.assets.len(), "Invalid asset id");

        balances[asset_id]
            .checked_sub(transaction.outputs.amounts[i])
            .unwrap();
    }

    assert_eq!(balances, [0u64; 4]);
}
