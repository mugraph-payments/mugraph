use mugraph_derive::SerializeBytes;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SerializeBytes)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Input {
    pub a: Note,
    pub b: Note,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, SerializeBytes)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Output {
    pub a: Hash,
    pub b: Hash,
    pub c: Hash,
}

pub type Context =
    crate::contracts::Context<{ Input::SIZE }, { BlindedNote::SIZE }, { Output::SIZE }>;

#[inline]
pub fn fusion(context: &mut Context) -> Result<()> {
    let Input { a: ia, b: ib } = context.read_stdin()?;

    assert_eq!(ia.asset_id, ib.asset_id);
    assert!(!ia.nullifier.is_empty());
    assert!(!ib.nullifier.is_empty());
    assert_ne!(ia.nullifier, ib.nullifier);

    let total = ia
        .amount
        .checked_add(ib.amount)
        .expect("overflow in total amount");

    let (a, b) = (
        Hash::digest(&mut context.hasher, &ia)?,
        Hash::digest(&mut context.hasher, &ib)?,
    );

    let output = BlindedNote {
        asset_id: ia.asset_id,
        amount: total,
        secret: Hash::combine3(&mut context.hasher, OUTPUT_SEP, a, b)?,
    };
    context.write_stdout(&output);

    let fusion = Output {
        a,
        b,
        c: Hash::digest(&mut context.hasher, &output)?,
    };
    context.write_journal(&fusion);

    Ok(())
}

#[cfg(all(feature = "std", test))]
mod tests {
    use super::*;

    use proptest::prelude::*;
    use test_strategy::proptest;

    #[proptest]
    fn test_fusion_success(
        a: Note,
        #[strategy(any::<Note>().prop_map(move |mut b| { b.asset_id = #a.asset_id; b }))] b: Note,
    ) {
        prop_assume!(a.amount.checked_add(b.amount).is_some());

        let mut context = Context::new();

        let input = Input {
            a: a.clone(),
            b: b.clone(),
        };
        input.to_slice(&mut context.stdin);

        fusion(&mut context)?;
        let result = BlindedNote::from_slice(&context.stdout)?;

        assert_eq!(a.asset_id, result.asset_id);
        assert_eq!(a.amount + b.amount, result.amount);
    }

    #[proptest]
    #[should_panic]
    fn test_fusion_overflow(
        #[strategy((u64::MAX / 2) + 1..u64::MAX)] _amount_a: u64,
        #[strategy((u64::MAX / 2) + 1..u64::MAX)] _amount_b: u64,
        #[strategy(any::<Note>().prop_map(move |mut a| { a.amount = #_amount_a; a }))] a: Note,
        #[strategy(any::<Note>().prop_map(move |mut b| {
            b.asset_id = #a.asset_id;
            b.amount = #_amount_b;
            b
        }))]
        b: Note,
    ) {
        let mut context = Context::new();

        let input = Input { a, b };
        input.to_slice(&mut context.stdin);

        fusion(&mut context)?;
    }

    #[proptest]
    #[should_panic]
    fn test_fusion_asset_id_mismatch(a: Note, b: Note) {
        prop_assume!(a.asset_id != b.asset_id);

        let mut context = Context::new();

        let input = Input { a, b };
        input.to_slice(&mut context.stdin);

        fusion(&mut context)?;
    }
}
