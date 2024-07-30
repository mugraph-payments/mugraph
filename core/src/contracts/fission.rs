use mugraph_derive::SerializeBytes;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::{contracts::Context, *};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SerializeBytes)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Input {
    pub server_key: PublicKey,
    pub input: Note,
    pub amount: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, SerializeBytes)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Output {
    pub a: Hash,
    pub b: Hash,
    pub c: Hash,
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
    context.write_stdout(&output);
    context.write_stdout(&change);

    Ok(())
}
