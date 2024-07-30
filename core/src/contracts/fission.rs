use core::ops::Range;

use crate::*;

pub const FISSION_TOTAL_SIZE: usize = Split::SIZE + BlindedNote::SIZE + Fission::SIZE;
pub const FISSION_STDIN_RANGE: Range<usize> = 0..Split::SIZE;
pub const FISSION_STDOUT_RANGE: Range<usize> = Split::SIZE..Split::SIZE + (BlindedNote::SIZE * 2);
pub const FISSION_JOURNAL_RANGE: Range<usize> = Split::SIZE + BlindedNote::SIZE..FISSION_TOTAL_SIZE;

#[inline(always)]
pub fn fission(memory: &mut [u8; FISSION_TOTAL_SIZE]) -> Result<()> {
    let request = Split::from_slice(&mut memory[FISSION_STDIN_RANGE])?;

    assert!(!request.input.nullifier.is_empty());
    assert_ne!(request.amount, 0);
    assert_ne!(request.input.amount, 0);
    assert!(request.input.amount >= request.amount);

    let input_hash = request.input.digest();

    let amount = request
        .input
        .amount
        .checked_sub(request.amount)
        .expect("input bigger than amount");

    let change = BlindedNote {
        asset_id: request.input.asset_id,
        amount,
        secret: Hash::combine3(input_hash, CHANGE_SEP, Hash::digest(&amount.to_le_bytes())?)?,
    };

    let amount = request
        .input
        .amount
        .checked_sub(change.amount)
        .expect("input bigger than amount");

    let output = BlindedNote {
        asset_id: request.input.asset_id,
        amount,
        secret: Hash::combine3(input_hash, OUTPUT_SEP, Hash::digest(&amount.to_le_bytes())?)?,
    };

    let stdout = &mut memory[FISSION_STDOUT_RANGE];

    output.to_slice(&mut stdout[..BlindedNote::SIZE]);
    change.to_slice(&mut stdout[BlindedNote::SIZE..]);

    let journal = &mut memory[FISSION_JOURNAL_RANGE];

    journal[..Hash::SIZE].copy_from_slice(&*input_hash);
    journal[Hash::SIZE..Hash::SIZE * 2].copy_from_slice(&*output.digest());
    journal[Hash::SIZE * 2..].copy_from_slice(&*change.digest());

    Ok(())
}
