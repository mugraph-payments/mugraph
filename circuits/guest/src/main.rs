#![no_main]
#![no_std]

use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
    // TODO: Implement your guest code here

    // read the input
    let a: Vec<u8> = env::read();

    // TODO: do something with the input
    assert_eq!(a, "hello", "Invalid Input");
}
