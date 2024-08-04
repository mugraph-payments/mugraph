#![no_std]

use mugraph_core_programs_guest::verify;
use risc0_zkvm::guest::env;

fn main() {
    verify(env::read()).unwrap();
}
