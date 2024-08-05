use mugraph_core::{
    error::{Error, Result},
    types::*,
};
use mugraph_core_programs::__build::{VALIDATE_ELF, VALIDATE_ID};
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

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let transaction = Transaction {
        manifest: Manifest {
            programs: ProgramSet {
                validate: VALIDATE_ID.into(),
            },
        },
        inputs: Inputs {
            parents: [[0; 32].into(); 4],
            indexes: [0, 1, 2, 3],
            asset_ids: [1, 2, 3, 1],
            amounts: [100, 200, 300, 400],
            program_id: [[0; 32].into(); 4],
            data: [u8::MAX; 4],
        },
        outputs: Outputs {
            asset_ids: [1, 2, 3, 1],
            amounts: [150, 200, 300, 350],
            program_id: [[0; 32].into(); 4],
            data: [u8::MAX; 4],
        },
        data: [0; 256 * 8],
        assets: [
            [1; 32].into(),
            [2; 32].into(),
            [3; 32].into(),
            [0; 32].into(),
        ],
    };

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
            .prove_with_opts(env, VALIDATE_ELF, &ProverOpts::succinct())
            .map_err(|_| Error::ZKVM)?
            .receipt
    );
    let cycles: u64 = risc0_zkvm::serde::from_slice(&stdout)?;
    info!("Done, proof took {} cycles.", cycles);

    Ok(())
}
