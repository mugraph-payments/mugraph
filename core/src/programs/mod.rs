use crate::types::*;

#[inline(always)]
#[no_mangle]
pub fn validate(transaction: Transaction) {
    let mut inputs = [0u128; MAX_INPUTS];
    let mut outputs = [0u128; MAX_INPUTS];

    for i in 0..MAX_ATOMS {
        let index = transaction.blob.asset_id_indexes[i] as usize;
        let amount = transaction.blob.amounts[i];
        let is_input = transaction.blob.parent_ids[i] != Hash::default();

        if is_input {
            inputs[index] += amount as u128;
        } else {
            outputs[index] += amount as u128;
        }
    }

    assert_eq!(inputs, outputs);
}
