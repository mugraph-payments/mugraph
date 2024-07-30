#![no_std]

use mugraph_core::{contracts::fission::*, Result};

use risc0_zkvm::guest::env;
use sha2::{Digest, Sha256};

fn main() -> Result<()> {
    let mut hasher = Sha256::new();
    let mut memory = [0u8; FISSION_TOTAL_SIZE];

    env::read_slice(&mut memory[FISSION_STDIN_RANGE]);

    fission(&mut hasher, &mut memory)?;

    env::write_slice(&mut memory[FISSION_STDOUT_RANGE]);
    env::commit_slice(&mut memory[FISSION_JOURNAL_RANGE]);

    Ok(())
}
