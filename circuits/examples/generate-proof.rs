use mugraph_circuits::{default_prover, swap::ELF, Error, ExecutorEnv, Result};
use mugraph_core::{Note, Transaction, TransactionBuilder};

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
        .map_err(|e| Error::FailedToWriteValue(e.to_string()))?
        .build()
        .map_err(|e| Error::FailedToInitializeExecutor(e.to_string()))?;

    let prover = default_prover();

    let receipt = prover.prove(env, ELF).unwrap().receipt;

    println!(
        "Hello, world! I generated a proof of guest execution!\nReceipt:\n\n{}",
        serde_json::to_string(&receipt).unwrap()
    );

    Ok(())
}
