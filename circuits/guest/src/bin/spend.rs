#![no_std]

use mugraph_core::{Hash, RequestSpend, Spend};
use risc0_zkvm::guest::env;

fn main() {
    let request: RequestSpend = env::read();

    assert!(!request.input.nullifier.is_empty());
    assert_ne!(request.amount, 0);
    assert_ne!(request.input.amount, 0);
    assert!(request.input.amount >= request.amount);

    let mut spend = Spend::default();
    spend.input = Hash::digest(&request.input.as_bytes()).unwrap();

    env::commit(&spend);
}
