#![no_std]

use mugraph_core::{BlindedNote, Fusion, Hash, Join, Result, OUTPUT_SEP};

use risc0_zkvm::guest::env;

#[inline(always)]
fn run() -> Result<()> {
    let request: Join = env::read();

    let [input_a, input_b] = request.inputs;

    assert!(!input_a.nullifier.is_empty());
    assert!(!input_b.nullifier.is_empty());

    assert_eq!(input_a.asset_id, input_b.asset_id);
    assert_ne!(input_a.nullifier, input_b.nullifier);

    assert_ne!(input_a.amount, 0);
    assert_ne!(input_b.amount, 0);

    let hash_a = Hash::digest(&input_a.as_bytes())?;
    let hash_b = Hash::digest(&input_b.as_bytes())?;

    let total_amount = input_a
        .amount
        .checked_add(input_b.amount)
        .expect("overflow in total amount");

    let output = BlindedNote {
        asset_id: input_a.asset_id,
        amount: total_amount,
        secret: Hash::combine4(
            hash_a,
            hash_b,
            OUTPUT_SEP,
            Hash::digest(&total_amount.to_le_bytes())?,
        )?,
    };

    env::write(&output);

    let fusion = Fusion {
        inputs: [hash_a, hash_b],
        output: Hash::digest(&output.as_bytes())?,
    };

    env::commit(&fusion);

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }
}
