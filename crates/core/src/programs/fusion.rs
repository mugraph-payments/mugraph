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

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, SerializeBytes)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Stdout {
    pub note: BlindedNote,
}

build_context_alias!(Input, Output, Stdout);

#[inline]
pub fn fusion(context: &mut Context) -> Result<()> {
    let Input { a: ia, b: ib } = context.read_stdin()?;

    assert_eq!(ia.asset_id, ib.asset_id);
    assert_eq!(ia.server_key, ib.server_key);
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

    let note = BlindedNote {
        server_key: ia.server_key,
        asset_id: ia.asset_id,
        amount: total,
        secret: Hash::combine3(&mut context.hasher, OUTPUT_SEP, a, b)?,
    };

    let output = Output {
        a,
        b,
        c: Hash::digest(&mut context.hasher, &note)?,
    };

    context.write_journal(&output);
    context.write_stdout(&Stdout { note });

    Ok(())
}

#[cfg(all(feature = "std", test))]
mod tests {
    use super::*;

    use proptest::prelude::*;
    use std::ops::Range;
    use test_strategy::proptest;

    fn pair() -> impl Strategy<Value = (Note, Note)> {
        pair_amount_range(0..u64::MAX / 2, 0..u64::MAX / 2).boxed()
    }

    fn pair_amount_range(ra: Range<u64>, rb: Range<u64>) -> impl Strategy<Value = (Note, Note)> {
        (any::<(Note, Note)>(), (ra, rb))
            .prop_map(|((mut a, mut b), (ra, rb))| {
                a.amount = ra;
                b.amount = rb;

                b.asset_id.copy_from_slice(&*a.asset_id);
                b.server_key.copy_from_slice(&*a.asset_id);

                (a, b)
            })
            .prop_filter("nullifiers must not be empty", |(a, b)| {
                !a.nullifier.is_empty() && !b.nullifier.is_empty()
            })
    }

    #[proptest]
    fn test_fusion_success(#[strategy(pair())] inputs: (Note, Note)) {
        let (a, b) = inputs;

        let mut context = Context::new();

        let input = Input {
            a: a.clone(),
            b: b.clone(),
        };
        context.write_stdin(&input);

        fusion(&mut context)?;
        let result = BlindedNote::from_slice(&context.stdout)?;

        assert_eq!(a.asset_id, result.asset_id);
        assert_eq!(a.amount + b.amount, result.amount);
    }

    #[proptest]
    #[should_panic]
    fn test_fusion_amount_overflow(
        #[strategy(pair_amount_range(u64::MAX / 2 + 1..u64::MAX, u64::MAX / 2 + 1..u64::MAX))]
        inputs: (Note, Note),
    ) {
        let (a, b) = inputs;
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
