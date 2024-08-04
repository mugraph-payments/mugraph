#![no_std]

use mugraph_core::programs::apply;
use risc0_zkvm::guest::env;

fn main() {
    apply(env::read()).unwrap();
}
