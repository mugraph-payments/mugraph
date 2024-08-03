use mugraph_core::{
    error::{Error, Result},
    types::*,
};
use mugraph_core_programs::__build::APPLY_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv};
use tracing::*;

macro_rules! timed {
    ($name:literal, $($arg:tt)*) => {{
        let s = tracing::span!(Level::INFO, concat!("mugraph::task[", $name, "]"));
        let _e = s.enter();

        tracing::debug!("Starting task");

        let now = std::time::Instant::now();
        let result = { $($arg)* };

        tracing::info!(elapsed = ?now.elapsed(), "Finished task");

        result
    }}
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let input = Operation::UNSAFE_Mint {
        output: Sealed {
            parent: [1u8; 32].into(),
            index: 0,
            data: Note {
                asset_id: [2u8; 32].into(),
                amount: 1337,
                program_id: None,
                sticky: false,
                datum: None,
            },
        },
    };

    let env = timed!("create executor", {
        ExecutorEnv::builder()
            .write(&input)
            .map_err(|e| Error::ZKVM(e.to_string()))?
            .build()
            .map_err(|e| Error::ZKVM(e.to_string()))?
    });

    // Obtain the default prover.
    let prover = timed!("create prover", default_prover());

    // Produce a receipt by proving the specified ELF binary.
    let receipt = timed!(
        "run prover",
        prover
            .prove(env, APPLY_ELF)
            .map_err(|e| Error::ZKVM(e.to_string()))?
            .receipt
    );

    let env = timed!("create executor", {
        ExecutorEnv::builder()
            .add_assumption(receipt)
            .write(&input)
            .map_err(|e| Error::ZKVM(e.to_string()))?
            .build()
            .map_err(|e| Error::ZKVM(e.to_string()))?
    });

    Ok(())
}
