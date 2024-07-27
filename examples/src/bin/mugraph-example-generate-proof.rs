use mugraph_circuits::*;
use mugraph_core::{Error, Note, Result, Swap, Transaction, TransactionBuilder};
use risc0_zkvm::{default_prover, ExecutorEnv};

fn main() -> Result<()> {
    let mut transaction = Transaction::default();
    let builder = TransactionBuilder::new(&mut transaction);
    builder
        .add_input(Note {
            asset_id: [1u8; 32],
            amount: 100,
            nullifier: [2u8; 32],
        })
        .unwrap()
        .add_output(Note {
            asset_id: [1u8; 32],
            amount: 50,
            nullifier: [5u8; 32],
        })
        .unwrap()
        .add_output(Note {
            asset_id: [1u8; 32],
            amount: 50,
            nullifier: [4u8; 32],
        })
        .unwrap()
        .build()
        .unwrap();

    let env = ExecutorEnv::builder()
        .write(&transaction)
        .map_err(|_| Error::FailedToWriteValue)?
        .build()
        .map_err(|_| Error::FailedToInitializeExecutor)?;

    let prover = default_prover();

    let receipt = prover.prove(env, SWAP_ELF).unwrap().receipt;
    let swap: Swap = receipt.journal.decode().unwrap();

    println!(
        "Hello, world! I generated a proof of guest execution!\nSwap:\n\n{}",
        serde_json::to_string_pretty(&swap).unwrap()
    );

    Ok(())
}
