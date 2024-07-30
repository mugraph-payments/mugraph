use core::ops::Range;

use crate::*;

pub const FUSION_TOTAL_SIZE: usize = Join::SIZE + BlindedNote::SIZE + Fusion::SIZE;
pub const FUSION_STDIN_RANGE: Range<usize> = 0..Join::SIZE;
pub const FUSION_STDOUT_RANGE: Range<usize> = Join::SIZE..Join::SIZE + BlindedNote::SIZE;
pub const FUSION_JOURNAL_RANGE: Range<usize> = Join::SIZE + BlindedNote::SIZE..FUSION_TOTAL_SIZE;

#[inline(always)]
pub fn fusion(memory: &mut [u8; FUSION_TOTAL_SIZE]) -> Result<()> {
    let join = Join::from_slice(&mut memory[FUSION_STDIN_RANGE])?;
    let [ia, ib] = join.inputs;
    let (a, b) = (ia.digest(), ib.digest());

    assert_eq!(ia.asset_id, ib.asset_id);
    assert!(!ia.nullifier.is_empty());
    assert!(!ib.nullifier.is_empty());
    assert_ne!(ia.nullifier, ib.nullifier);

    let total = ia
        .amount
        .checked_add(ib.amount)
        .expect("overflow in total amount");

    let output = BlindedNote {
        asset_id: ia.asset_id,
        amount: total,
        secret: Hash::combine3(OUTPUT_SEP, a, b)?,
    };

    output.to_slice(&mut memory[FUSION_STDOUT_RANGE]);

    let fusion = Fusion {
        a,
        b,
        c: output.digest(),
    };

    fusion.to_slice(&mut memory[FUSION_JOURNAL_RANGE]);

    Ok(())
}
