use mugraph_client::prelude::*;
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};
use tracing::*;

macro_rules! timed {
    ($name:literal, $($arg:tt)*) => {{
        let s = tracing::span!(Level::INFO, concat!("mugraph::task[", $name, "]"));
        let _e = s.enter();

        tracing::info!("Starting task");

        let now = std::time::Instant::now();
        let result = { $($arg)* };

        tracing::info!(elapsed = ?now.elapsed(), "Finished task");

        result
    }}
}

fn build_transaction() -> Transaction {
    let manifest = Manifest {
        programs: ProgramSet {
            validate: VALIDATE_ID.into(),
        },
    };

    let note = Note {
        parent_id: Hash::digest(b"Parent Id"),
        asset_id: Hash::digest(b"Sample Asset"),
        nonce: Hash::digest(b"Sample Nonce"),
        amount: 100,
    };

    TransactionBuilder::new(manifest)
        .input(&note)
        .output(note.asset_id, 60)
        .output(note.asset_id, 40)
        .build()
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let transaction = build_transaction();

    let mut stdout = Vec::new();

    let env = timed!("create executor", {
        ExecutorEnv::builder()
            .write(&transaction)
            .map_err(|_| Error::ZKVM)?
            .stdout(&mut stdout)
            .build()
            .map_err(|_| Error::ZKVM)?
    });

    // Obtain the default prover.
    let prover = timed!("create prover", default_prover());

    // Produce a receipt by proving the specified ELF binary.
    let _ = timed!(
        "prove transaction",
        prover
            .prove_with_opts(env, VALIDATE_ELF, &ProverOpts::fast())
            .map_err(|_| Error::ZKVM)?
            .receipt
    );
    let cycles: u64 = risc0_zkvm::serde::from_slice(&stdout)?;
    info!("Done, proof took {} cycles.", cycles);

    Ok(())
}
