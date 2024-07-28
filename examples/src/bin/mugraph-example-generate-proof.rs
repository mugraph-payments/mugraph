use mugraph_circuits::*;
use mugraph_core::{Error, Hash, Note, RequestSpend, Result, Spend};
use risc0_zkvm::{default_prover, serde::from_slice, ExecutorEnv, ProverOpts};

fn main() -> Result<()> {
    let request = RequestSpend {
        input: Note {
            asset_id: Hash([1u8; 32]),
            amount: 100,
            nullifier: Hash([2u8; 32]),
        },
        amount: 50,
    };

    let mut buf = Vec::new();

    let env = ExecutorEnv::builder()
        .write(&request)
        .map_err(|_| Error::ExecutorWriteValue)?
        .stdout(&mut buf)
        .build()
        .map_err(|_| Error::ExecutorInitialize)?;

    let prover = default_prover();

    let receipt = prover
        .prove_with_opts(env, SPEND_ELF, &ProverOpts::succinct())
        .map_err(|_| Error::ProofGenerate)?
        .receipt;
    let spend: Spend = receipt.journal.decode().map_err(|_| Error::JournalDecode)?;
    let (output, change): (Note, Note) = from_slice(&buf).map_err(|_| Error::StdoutDecode)?;

    println!(
        "Spend:\n\n{}",
        serde_json::to_string_pretty(&spend).unwrap()
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
