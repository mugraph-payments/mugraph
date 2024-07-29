use mugraph_core::{BlindedNote, Hash, Result, Split, CHANGE_SEP, OUTPUT_SEP};

use risc0_zkvm::guest::env;

#[inline(always)]
fn run() -> Result<()> {
    let mut buf = [0u8; Split::SIZE];
    let mut out = [0u8; BlindedNote::SIZE];
    let mut chan = [0u8; BlindedNote::SIZE];

    env::read_slice(&mut buf);

    let request = Split::from_bytes(&buf)?;

    assert!(!request.input.nullifier.is_empty());
    assert_ne!(request.amount, 0);
    assert_ne!(request.input.amount, 0);
    assert!(request.input.amount >= request.amount);

    let input_hash = request.input.digest();

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
    change.to_slice(&mut chan);

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
    output.to_slice(&mut out);

    env::commit_slice(&[*input_hash, *output.digest(), *change.digest()].concat());
    env::write_slice(&[out, chan].concat());

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }
}
