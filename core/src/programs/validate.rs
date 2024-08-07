use crate::types::*;

static mut INPUTS: [u128; MAX_INPUTS] = [0; MAX_INPUTS];
static mut OUTPUTS: [u128; MAX_INPUTS] = [0; MAX_INPUTS];

#[inline(always)]
#[no_mangle]
pub fn validate(transaction: &Transaction) {
    for i in 0..MAX_ATOMS {
        let index = transaction.asset_id_indexes[i] as usize;
        let amount = transaction.amounts[i];
        let is_input = transaction.input_mask.contains(i as u8);

        if is_input {
            unsafe {
                INPUTS[index] += amount as u128;
            }
        } else {
            unsafe {
                OUTPUTS[index] += amount as u128;
            }
        }
    }

    unsafe {
        assert_eq!(INPUTS, OUTPUTS);
    }
}
