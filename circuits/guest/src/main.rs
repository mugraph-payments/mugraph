use risc0_zkvm::guest::env;

fn main() {
    let input: u32 = env::read();

    // TODO: do something with the input

    // write public output to the journal
    env::commit(&input);
}
