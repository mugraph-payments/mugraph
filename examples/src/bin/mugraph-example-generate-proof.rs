use mugraph_circuits::*;
use mugraph_core::{Error, Fission, Hash, Note, Result, Split};
use risc0_zkvm::{default_prover, serde::from_slice, ExecutorEnv, ProverOpts};

fn main() -> Result<()> {
    let request = Split {
        input: Note {
            asset_id: Hash([1u8; 32]),
            amount: 100,
            nullifier: Hash([2u8; 32]),
        },
        amount: 50,
    };

    let mut stdout = Vec::new();

    let env = ExecutorEnv::builder()
        .write(&request)
        .map_err(|_| Error::ExecutorWriteValue)?
        .stdout(&mut stdout)
        .build()
        .map_err(|_| Error::ExecutorInitialize)?;

    let prover = default_prover();

    let receipt = prover
        .prove_with_opts(env, FISSION_ELF, &ProverOpts::succinct())
        .map_err(|e| {
            println!("Error: {}", e);
            Error::ProofGenerate
        })?
        .receipt;
    let fission: Fission = receipt.journal.decode().map_err(|_| Error::JournalDecode)?;
    let (output, change): (Note, Note) = from_slice(&stdout).map_err(|_| Error::StdoutDecode)?;

    println!(
        "Spend:\n\n{}",
        serde_json::to_string_pretty(&fission).unwrap()
    );
    println!(
        "Output:\n\n{}",
        serde_json::to_string_pretty(&output).unwrap()
    );
    println!(
        "Change:\n\n{}",
        serde_json::to_string_pretty(&change).unwrap()
    );

    Ok(())
}
