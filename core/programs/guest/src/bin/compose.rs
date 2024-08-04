#![no_std]

use mugraph_core::programs::compose;
use risc0_zkvm::guest::env;

fn main() {
    compose(env::read()).unwrap();
}
