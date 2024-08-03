use mugraph_core::{error::Result, types::Operation};
use mugraph_core_programs_guest::verify;
use risc0_zkvm::guest::env;

fn main() -> Result<()> {
    let op: Operation = env::read();
    verify(&op)?;

    Ok(())
}
