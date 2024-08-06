#![no_std]

use mugraph_core::programs::validate;
use risc0_zkvm::guest::env;

fn main() {
    validate(env::read()).unwrap();
    env::write_slice(&env::cycle_count().to_le_bytes());
}
