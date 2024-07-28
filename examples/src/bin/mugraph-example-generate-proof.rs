use mugraph_circuits::*;
use mugraph_core::{Error, Note, RequestSpend, Result, Spend};
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};

fn main() -> Result<()> {
    let request = RequestSpend {
        input: Note {
            asset_id: [1u8; 32],
            amount: 100,
            nullifier: [2u8; 32],
        },
        amount: 50,
    };

    let env = ExecutorEnv::builder()
        .write(&request)
        .map_err(|_| Error::ExecutorWriteValue)?
        .build()
        .map_err(|_| Error::ExecutorInitialize)?;

    let prover = default_prover();

    let receipt = prover
        .prove_with_opts(env, SPEND_ELF, &ProverOpts::succinct())
        .map_err(|_| Error::ProofGenerate)?
        .receipt;
    let spend: Spend = receipt.journal.decode().map_err(|_| Error::JournalDecode)?;

    println!(
        "Hello, world! I generated a proof of guest execution!\nSpend:\n\n{}\n\nReceipt JSON size:\n\n{}",
        serde_json::to_string_pretty(&spend).unwrap(),
        serde_json::to_string_pretty(&receipt).unwrap().len()
    );

    Ok(())
}
