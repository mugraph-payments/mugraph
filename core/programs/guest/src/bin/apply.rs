#![no_std]

use mugraph_core_programs_guest::verify;
use risc0_zkvm::guest::env;

fn main() {
    let op = env::read();
    verify(&op).unwrap();
}
