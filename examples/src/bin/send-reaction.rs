use mugraph_core::{
    error::{Error, Result},
    types::*,
};
use mugraph_core_programs::__build::{APPLY_ELF, APPLY_ID, COMPOSE_ID};
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

    let manifest = Manifest {
        programs: ProgramSet {
            apply: APPLY_ID.into(),
            compose: COMPOSE_ID.into(),
        },
    };

    let sealed_note = Sealed {
        parent: [1u8; 32].into(),
        index: 0,
        data: Note {
            asset_id: [2u8; 32].into(),
            amount: 1337,
            program_id: None,
            datum: None,
        },
    };
    let mint = Operation::UNSAFE_Mint {
        output: sealed_note.clone(),
    };

    let env = timed!("create executor", {
        ExecutorEnv::builder()
            .write(&Request {
                manifest: manifest.clone(),
                data: mint,
            })
            .map_err(|e| Error::ZKVM(e.to_string()))?
            .build()
            .map_err(|e| Error::ZKVM(e.to_string()))?
    });

    // Obtain the default prover.
    let prover = timed!("create prover", default_prover());

    // Produce a receipt by proving the specified ELF binary.
    let mint_receipt = timed!(
        "prove mint",
        prover
            .prove_with_opts(env, APPLY_ELF, &ProverOpts::fast())
            .map_err(|e| Error::ZKVM(e.to_string()))?
            .receipt
    );

    let consume = Operation::Consume {
        input: sealed_note,
        output: Note {
            asset_id: [2u8; 32].into(),
            amount: 1337,
            program_id: None,
            datum: None,
        },
    };

    let env = timed!("create executor", {
        ExecutorEnv::builder()
            .add_assumption(mint_receipt)
            .write(&Request {
                manifest,
                data: consume,
            })
            .map_err(|e| Error::ZKVM(e.to_string()))?
            .build()
            .map_err(|e| Error::ZKVM(e.to_string()))?
    });

    // Produce a receipt by proving the specified ELF binary.
    let consume_receipt = timed!(
        "prove consume",
        prover
            .prove_with_opts(env, APPLY_ELF, &ProverOpts::fast())
            .map_err(|e| Error::ZKVM(e.to_string()))?
            .receipt
    );

    let _compressed = timed!(
        "prove compress",
        prover.compress(&ProverOpts::groth16(), &consume_receipt)
    );

    println!("Ok");

    Ok(())
}
