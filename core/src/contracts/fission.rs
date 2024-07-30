use contracts::Context;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::*;

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
        debug_assert!(out.len() >= Self::SIZE);

        self.server_key.to_slice(&mut out[..32]);
        self.input.to_slice(&mut out[32..32 + Note::SIZE]);
        self.amount.to_slice(&mut out[32 + Note::SIZE..Self::SIZE]);
    }

    #[inline]
    fn from_slice(input: &[u8]) -> Result<Self> {
        debug_assert!(input.len() >= Self::SIZE);

        Ok(Self {
            server_key: PublicKey::from_slice(&input[..32])?,
            input: Note::from_slice(&input[32..32 + Note::SIZE])?,
            amount: u64::from_slice(&input[32 + Note::SIZE..Self::SIZE])?,
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
        debug_assert!(out.len() >= Self::SIZE);

        self.a.to_slice(&mut out[..32]);
        self.b.to_slice(&mut out[32..64]);
        self.c.to_slice(&mut out[64..Self::SIZE]);
    }

    #[inline]
    fn from_slice(input: &[u8]) -> Result<Self> {
        debug_assert!(input.len() >= Self::SIZE);

        Ok(Self {
            a: Hash::from_slice(&input[..32])?,
            b: Hash::from_slice(&input[32..64])?,
            c: Hash::from_slice(&input[64..Self::SIZE])?,
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

    let input_hash = request.input.digest(hasher);

    let amount = request
        .input
        .amount
        .checked_sub(request.amount)
        .expect("input bigger than amount");
    let amount_digest = amount.digest(hasher);
    let request_amount_digest = request.amount.digest(hasher);

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
        b: output.digest(hasher),
        c: change.digest(hasher),
    };

    context.write_journal(&fission);
    context.write_stdout(&(output, change));

    Ok(())
}
