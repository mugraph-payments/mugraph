use mugraph_core::{BlindedNote, Hash, Result, Split, CHANGE_SEP, OUTPUT_SEP};

use risc0_zkvm::guest::env;

#[inline(always)]
fn run() -> Result<()> {
    let mut buf = [0u8; Split::SIZE];
    env::read_slice(&mut buf);

    let request = Split::from_bytes(&buf)?;

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

    let change = BlindedNote {
        asset_id: request.input.asset_id,
        amount,
        secret: Hash::combine3(input_hash, CHANGE_SEP, Hash::digest(&amount.to_le_bytes())?)?,
    };

    let amount = request
        .input
        .amount
        .checked_sub(change.amount)
        .expect("input bigger than amount");

    let output = BlindedNote {
        asset_id: request.input.asset_id,
        amount,
        secret: Hash::combine3(input_hash, OUTPUT_SEP, Hash::digest(&amount.to_le_bytes())?)?,
    };

    env::commit_slice(
        &[
            *input_hash,
            *Hash::digest(&output.as_bytes())?,
            *Hash::digest(&change.as_bytes())?,
        ]
        .concat(),
    );
    env::write(&(output, change));

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }
}
