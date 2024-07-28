#![no_std]

use mugraph_core::{Hash, RequestSpend, Result, Spend, CHANGE_SEP, OUTPUT_SEP};
use risc0_zkvm::guest::env;

fn main() -> Result<()> {
    let request: RequestSpend = env::read();

    assert!(!request.input.nullifier.is_empty());
    assert_ne!(request.amount, 0);
    assert_ne!(request.input.amount, 0);
    assert!(request.input.amount >= request.amount);

    let mut spend = Spend::default();

    spend.input = Hash::digest(&request.input.as_bytes()).unwrap();
    spend.outputs[0] = Hash::combine3(
        spend.input,
        Hash::digest(&OUTPUT_SEP).unwrap(),
        Hash::digest(&request.amount.to_le_bytes()).unwrap(),
    )?;

    let change = request
        .input
        .amount
        .checked_sub(request.amount)
        .expect("input bigger than amount");

    spend.outputs[1] = Hash::combine3(
        spend.input,
        Hash::digest(&CHANGE_SEP).unwrap(),
        Hash::digest(&change.to_le_bytes()).unwrap(),
    )?;

    env::commit(&spend);

    Ok(())
}
