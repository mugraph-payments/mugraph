#![no_std]

use mugraph_core::{programs::fission::*, Result};
use risc0_zkvm::guest::env;

fn main() -> Result<()> {
    let mut context = Context::new();

    env::read_slice(&mut context.stdin);

    fission(&mut context)?;

    env::write_slice(&context.stdout);
    env::commit_slice(&context.journal);

    Ok(())
}
