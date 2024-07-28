#![no_std]

use mugraph_core::{Hash, Note, RequestSpend, Result, Spend, CHANGE_SEP, OUTPUT_SEP};
use risc0_zkvm::guest::env;

fn main() -> Result<()> {
    let request: RequestSpend = env::read();

    assert!(!request.input.nullifier.is_empty());
    assert_ne!(request.amount, 0);
    assert_ne!(request.input.amount, 0);
    assert!(request.input.amount >= request.amount);

    let input_hash = Hash::digest(&request.input.as_bytes())?;

    let amount = request
        .input
        .amount
        .checked_sub(request.amount)
        .expect("input bigger than amount");

    let change = Note {
        asset_id: request.input.asset_id,
        amount,
        nullifier: Hash::combine3(
            input_hash,
            Hash::digest(&CHANGE_SEP).unwrap(),
            Hash::digest(&amount.to_le_bytes()).unwrap(),
        )?,
    };

    let amount = request
        .input
        .amount
        .checked_sub(change.amount)
        .expect("input bigger than amount");

    let output = Note {
        asset_id: request.input.asset_id,
        amount,
        nullifier: Hash::combine3(
            input_hash,
            Hash::digest(&OUTPUT_SEP).unwrap(),
            Hash::digest(&amount.to_le_bytes()).unwrap(),
        )?,
    };

    let spend = Spend {
        input: input_hash,
        outputs: [
            Hash::digest(&output.as_bytes())?,
            Hash::digest(&change.as_bytes())?,
        ],
    };

    env::write(&(output, change));
    env::commit(&spend);

    Ok(())
}
