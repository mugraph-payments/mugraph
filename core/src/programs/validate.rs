use crate::types::*;

#[inline(always)]
#[no_mangle]
pub fn validate(transaction: &Transaction) {
    let mut amounts = [0u64; 4];

    for i in 0..4 {
        assert_ne!(transaction.inputs.amounts[i], 0);

        let index = transaction
            .assets
            .iter()
            .position(|a| a[0] == transaction.inputs.asset_ids[i])
            .unwrap_or(0);

        amounts[index] = amounts[index]
            .checked_add(transaction.inputs.amounts[i])
            .unwrap();
    }

    for i in 0..4 {
        let index = transaction
            .assets
            .iter()
            .position(|a| a[0] == transaction.outputs.asset_ids[i])
            .unwrap_or(0);

        amounts[index] = amounts[index]
            .checked_sub(transaction.outputs.amounts[i])
            .unwrap();
    }

    assert_eq!(amounts, amounts);
}
