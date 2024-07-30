#![no_std]

use mugraph_core::{contracts::fusion::*, Result};

use risc0_zkvm::guest::env;
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let mut hasher = Sha256::new();
    let mut memory = [0u8; FUSION_TOTAL_SIZE];

    env::read_slice(&mut memory[FUSION_STDIN_RANGE]);

    fusion(&mut hasher, &mut memory)?;

    env::write_slice(&mut memory[FUSION_STDOUT_RANGE]);
    env::commit_slice(&mut memory[FUSION_JOURNAL_RANGE]);

    Ok(())
}
