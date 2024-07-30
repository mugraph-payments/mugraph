#![no_std]

use mugraph_core::{BlindedNote, Hash, Join, Result, OUTPUT_SEP};

use risc0_zkvm::guest::env;

fn main() -> Result<()> {
    let mut buf = [0u8; Join::SIZE];
    env::read_slice(&mut buf);

    let join = Join::from_bytes(&buf)?;
    let [input_a, input_b] = join.inputs;

    assert_eq!(input_a.asset_id, input_b.asset_id);
    assert!(!input_a.nullifier.is_empty());
    assert!(!input_b.nullifier.is_empty());
    assert_ne!(input_a.nullifier, input_b.nullifier);

    let a = input_a.digest();
    let b = input_b.digest();

    let total = input_a
        .amount
        .checked_add(input_b.amount)
        .expect("overflow in total amount");

    let output = BlindedNote {
        asset_id: input_a.asset_id,
        amount: total,
        secret: Hash::combine3(OUTPUT_SEP, a, b)?,
    };

    env::write(&output);

    env::commit_slice(&[*a, *b, *output.digest()].concat());

    Ok(())
}
