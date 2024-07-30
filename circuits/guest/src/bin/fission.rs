#![no_std]

use mugraph_core::{
    contracts::{fission::*, Context},
    BlindedNote, Result, SerializeBytes,
};

use risc0_zkvm::guest::env;
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let mut hasher = Sha256::new();
    let mut context =
        Context::<{ Input::SIZE }, { <(BlindedNote, BlindedNote)>::SIZE }, { Output::SIZE }>::new();

    env::read_slice(&mut context.stdin);

    fission(&mut hasher, &mut context)?;

    env::write_slice(&context.stdout);
    env::commit_slice(&context.journal);

    Ok(())
}
