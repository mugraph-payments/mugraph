use core::ops::Range;

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
        self.server_key.to_slice(&mut out[..32]);
        self.input.to_slice(&mut out[32..32 + Note::SIZE]);
        self.amount.to_slice(&mut out[32 + Note::SIZE..]);
    }

    #[inline]
    fn from_slice(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(crate::Error::FailedDeserialization);
        }

        Ok(Self {
            server_key: PublicKey::from_slice(&bytes[..32])?,
            input: Note::from_slice(&bytes[32..32 + Note::SIZE])?,
            amount: u64::from_slice(&bytes[32 + Note::SIZE..])?,
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
        self.a.to_slice(&mut out[..Hash::SIZE]);
        self.b.to_slice(&mut out[Hash::SIZE..Hash::SIZE * 2]);
        self.c.to_slice(&mut out[Hash::SIZE * 2..Hash::SIZE * 3]);
    }

    #[inline]
    fn from_slice(input: &[u8]) -> Result<Self> {
        Ok(Self {
            a: Hash::from_slice(&input[..Hash::SIZE])?,
            b: Hash::from_slice(&input[Hash::SIZE..Hash::SIZE * 2])?,
            c: Hash::from_slice(&input[Hash::SIZE * 2..Hash::SIZE * 3])?,
        })
    }
}

pub const FISSION_TOTAL_SIZE: usize = Input::SIZE + BlindedNote::SIZE + Output::SIZE;
pub const FISSION_STDIN_RANGE: Range<usize> = 0..Input::SIZE;
pub const FISSION_STDOUT_RANGE: Range<usize> = Input::SIZE..Input::SIZE + (BlindedNote::SIZE * 2);
pub const FISSION_JOURNAL_RANGE: Range<usize> = Input::SIZE + BlindedNote::SIZE..FISSION_TOTAL_SIZE;

#[inline]
pub fn fission(hasher: &mut Sha256, memory: &mut [u8; FISSION_TOTAL_SIZE]) -> Result<()> {
    let request = Input::from_slice(&mut memory[FISSION_STDIN_RANGE])?;

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

    let stdout = &mut memory[FISSION_STDOUT_RANGE];

    output.to_slice(&mut stdout[..BlindedNote::SIZE]);
    change.to_slice(&mut stdout[BlindedNote::SIZE..]);

    let journal = &mut memory[FISSION_JOURNAL_RANGE];

    journal[..Hash::SIZE].copy_from_slice(&*input_hash);
    journal[Hash::SIZE..Hash::SIZE * 2].copy_from_slice(&*output.digest(hasher));
    journal[Hash::SIZE * 2..].copy_from_slice(&*change.digest(hasher));

    Ok(())
}
