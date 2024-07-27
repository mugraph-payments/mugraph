use mugraph_circuits::{default_prover, swap::ELF, Error, ExecutorEnv, Result};
use mugraph_core::Note;

fn main() -> Result<()> {
    let note = Note {
        asset_id: [1u8; 32],
        amount: 100,
        nullifier: [2u8; 32],
    };

    let env = ExecutorEnv::builder()
        .write(&note)
        .map_err(|e| Error::FailedToWriteValue(e.to_string()))?
        .build()
        .map_err(|e| Error::FailedToInitializeExecutor(e.to_string()))?;

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover.prove(env, ELF).unwrap().receipt;

    println!("{}", serde_json::to_string_pretty(&receipt).unwrap());

    // Extract journal of receipt
    let output: u32 = receipt.journal.decode().unwrap();

    // Print, notice, after committing to a journal, the private input became public
    println!(
        "Hello, world! I generated a proof of guest execution! {} is a public output from journal ",
        output
    );

    Ok(())
}
