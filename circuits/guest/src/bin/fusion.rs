#![no_std]

use mugraph_core::{contracts::fusion::*, Result};

use risc0_zkvm::guest::env;

fn main() -> Result<()> {
    let mut memory = [0u8; FUSION_TOTAL_SIZE];

    env::read_slice(&mut memory[FUSION_STDIN_RANGE]);

    fusion(&mut memory)?;

    env::write_slice(&mut memory[FUSION_STDOUT_RANGE]);
    env::commit_slice(&mut memory[FUSION_JOURNAL_RANGE]);

    Ok(())
}
