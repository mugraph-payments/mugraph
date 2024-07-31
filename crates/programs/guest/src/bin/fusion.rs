#![no_std]

use mugraph_core::{programs::fusion::*, Result};
use risc0_zkvm::guest::env;

fn main() -> Result<()> {
    let mut context = Context::new();

    env::read_slice(&mut context.stdin);

    fusion(&mut context)?;

    env::write_slice(&context.stdout);
    env::commit_slice(&context.journal);

    Ok(())
}
