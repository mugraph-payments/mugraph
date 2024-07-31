use mugraph_derive::SerializeBytes;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, SerializeBytes)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Input {
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

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, SerializeBytes)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Stdout {
    pub output: BlindedNote,
    pub change: BlindedNote,
}

build_contract_alias!(Input, Output, Stdout);

#[inline]
pub fn fission(ctx: &mut Context) -> Result<()> {
    let request: Input = ctx.read_stdin()?;

    assert!(!request.input.nullifier.is_empty());
    assert_ne!(request.amount, 0);
    assert_ne!(request.input.amount, 0);
    assert!(request.input.amount >= request.amount);

    let input_hash = Hash::digest(&mut ctx.hasher, &request.input)?;

    let amount = request
        .input
        .amount
        .checked_sub(request.amount)
        .expect("input bigger than amount");
    let amount_digest = Hash::digest(&mut ctx.hasher, &amount)?;
    let request_amount_digest = Hash::digest(&mut ctx.hasher, &request.amount)?;

    let stdout = Stdout {
        output: BlindedNote {
            asset_id: request.input.asset_id,
            server_key: request.input.server_key,
            amount: request.amount,
            secret: Hash::combine3(
                &mut ctx.hasher,
                input_hash,
                OUTPUT_SEP,
                request_amount_digest,
            )?,
        },
        change: BlindedNote {
            asset_id: request.input.asset_id,
            server_key: request.input.server_key,
            amount,
            secret: Hash::combine3(&mut ctx.hasher, input_hash, CHANGE_SEP, amount_digest)?,
        },
    };

    let fission = Output {
        a: input_hash,
        b: Hash::digest(&mut ctx.hasher, &stdout.output)?,
        c: Hash::digest(&mut ctx.hasher, &stdout.change)?,
    };

    ctx.write_journal(&fission);
    ctx.write_stdout(&stdout);

    Ok(())
}
