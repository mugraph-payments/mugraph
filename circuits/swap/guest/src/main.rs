use mugraph_core::Note;
use risc0_zkvm::guest::env;

fn main() {
    let input: Note = env::read();

    env::commit(&input.amount);
}
