use mugraph_circuits::{default_prover, swap::ELF, Error, ExecutorEnv, Result};
use mugraph_core::{Note, Transaction};

fn main() -> Result<()> {
    let transaction = Transaction {
        inputs: [
            Some(Note {
                asset_id: [1u8; 32],
                amount: 100,
                nullifier: [2u8; 32],
            }),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
        outputs: [
            Some(Note {
                asset_id: [1u8; 32],
                amount: 50,
                nullifier: [5u8; 32],
            }),
            Some(Note {
                asset_id: [1u8; 32],
                amount: 50,
                nullifier: [4u8; 32],
            }),
            None,
            None,
            None,
            None,
            None,
            None,
        ],
    };

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
