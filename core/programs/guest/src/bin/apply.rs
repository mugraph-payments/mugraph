#![no_std]

use mugraph_core::programs::verify;
use risc0_zkvm::guest::env;

fn main() {
    verify(env::read()).unwrap();
}
