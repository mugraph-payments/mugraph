use risc0_zkvm::guest::env;

fn main() {
    let result: u8 = env::read();
    println!("{}", result);
}
