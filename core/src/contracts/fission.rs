use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::{contracts::Context, *};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Input {
    pub server_key: PublicKey,
    pub input: Note,
    pub amount: u64,
}

impl SerializeBytes for Input {
    const SIZE: usize = 32 + Note::SIZE + 8;

    #[inline]
    fn to_slice(&self, out: &mut [u8]) {
        let mut w = Writer::new(out);

        w.write(&self.server_key);
        w.write(&self.input);
        w.write(&self.amount);
    }

    #[inline]
    fn from_slice(input: &[u8]) -> Result<Self> {
        let mut r = Reader::new(input);

        Ok(Self {
            server_key: r.read()?,
            input: r.read()?,
            amount: r.read()?,
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
pub fn fission(
    hasher: &mut Sha256,
    context: &mut Context<{ Input::SIZE }, { BlindedNote::SIZE * 2 }, { Output::SIZE }>,
) -> Result<()> {
    let request: Input = context.read_stdin()?;

    assert!(!request.input.nullifier.is_empty());
    assert_ne!(request.amount, 0);
    assert_ne!(request.input.amount, 0);
    assert!(request.input.amount >= request.amount);

    let input_hash = Hash::digest(hasher, &request.input)?;

    let amount = request
        .input
        .amount
        .checked_sub(request.amount)
        .expect("input bigger than amount");
    let amount_digest = Hash::digest(hasher, &amount)?;
    let request_amount_digest = Hash::digest(hasher, &request.amount)?;

    let change = BlindedNote {
        asset_id: request.input.asset_id,
        amount,
        secret: Hash::combine3(hasher, input_hash, CHANGE_SEP, amount_digest)?,
    };

    let output = BlindedNote {
        asset_id: request.input.asset_id,
        amount: request.amount,
        secret: Hash::combine3(hasher, input_hash, OUTPUT_SEP, request_amount_digest)?,
    };

    let fission = Output {
        a: input_hash,
        b: Hash::digest(hasher, &output)?,
        c: Hash::digest(hasher, &change)?,
    };

    context.write_journal(&fission);
    context.write_stdout(&(output, change));

    Ok(())
}
