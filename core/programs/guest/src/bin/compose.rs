#![no_std]

use mugraph_core_programs_guest::compose;
use risc0_zkvm::guest::env;

fn main() {
    compose(env::read()).unwrap();
}
