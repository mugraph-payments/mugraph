#![no_std]

use mugraph_core::programs::validate;
use risc0_zkvm::guest::env;

fn main() {
    validate(&env::read());
    env::write(&env::cycle_count());
}
