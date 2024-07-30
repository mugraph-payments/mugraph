use contracts::Context;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Input {
    pub inputs: [Note; 2],
}

impl SerializeBytes for Input {
    const SIZE: usize = 2 * Note::SIZE;

    #[inline]
    fn to_slice(&self, out: &mut [u8]) {
        let mut w = Writer::new(out);
        w.write(&self.inputs[0]);
        w.write(&self.inputs[1]);
    }

    #[inline]
    fn from_slice(input: &[u8]) -> Result<Self> {
        let mut r = Reader::new(input);
        Ok(Self {
            inputs: [r.read()?, r.read()?],
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Output {
    pub a: Hash,
    pub b: Hash,
    pub c: Hash,
}

impl SerializeBytes for Output {
    const SIZE: usize = 3 * 32;

    #[inline]
    fn to_slice(&self, out: &mut [u8]) {
        let mut w = Writer::new(out);
        w.write(&self.a);
        w.write(&self.b);
        w.write(&self.c);
    }

    #[inline]
    fn from_slice(input: &[u8]) -> Result<Self> {
        let mut r = Reader::new(input);
        Ok(Self {
            a: r.read()?,
            b: r.read()?,
            c: r.read()?,
        })
    }
}

#[inline]
pub fn fusion(
    hasher: &mut Sha256,
    context: &mut Context<{ Input::SIZE }, { BlindedNote::SIZE }, { Output::SIZE }>,
) -> Result<()> {
    let input: Input = context.read_stdin()?;
    let [ia, ib] = input.inputs;
    let (a, b) = (Hash::digest(hasher, &ia)?, Hash::digest(hasher, &ib)?);

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
        secret: Hash::combine3(hasher, OUTPUT_SEP, a, b)?,
    };
    context.write_stdout(&output);

    let fusion = Output {
        a,
        b,
        c: Hash::digest(hasher, &output)?,
    };
    context.write_journal(&fusion);

    Ok(())
}

#[cfg(all(feature = "std", test))]
mod tests {
    use super::*;

    use proptest::prelude::*;
    use sha2::Digest;
    use test_strategy::proptest;

    #[proptest]
    fn test_fusion_success(
        a: Note,
        #[strategy(any::<Note>().prop_map(move |mut b| { b.asset_id = #a.asset_id; b }))] b: Note,
    ) {
        prop_assume!(a.amount.checked_add(b.amount).is_some());

        let mut hasher = Sha256::new();
        let mut context =
            Context::<{ Input::SIZE }, { BlindedNote::SIZE }, { Output::SIZE }>::new();

        Input {
            inputs: [a.clone(), b.clone()],
        }
        .to_slice(&mut context.stdin);

        fusion(&mut hasher, &mut context)?;
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
        let mut hasher = Sha256::new();
        let mut context =
            Context::<{ Input::SIZE }, { BlindedNote::SIZE }, { Output::SIZE }>::new();

        Input {
            inputs: [a.clone(), b.clone()],
        }
        .to_slice(&mut context.stdin);

        fusion(&mut hasher, &mut context)?;
    }

    #[proptest]
    #[should_panic]
    fn test_fusion_asset_id_mismatch(a: Note, b: Note) {
        prop_assume!(a.asset_id != b.asset_id);

        let mut hasher = Sha256::new();
        let mut context =
            Context::<{ Input::SIZE }, { BlindedNote::SIZE }, { Output::SIZE }>::new();

        Input {
            inputs: [a.clone(), b.clone()],
        }
        .to_slice(&mut context.stdin);

        fusion(&mut hasher, &mut context)?;
    }
}
