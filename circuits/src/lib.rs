use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, Receipt};
use serde::{de::DeserializeOwned, Serialize};

mod error;

pub use self::error::{Error, Result};

include!(concat!(env!("OUT_DIR"), "/methods.rs"));

pub struct Prover {
    stdout: Vec<u8>,
    opts: ProverOpts,
}

impl Prover {
    pub fn new() -> Self {
        Self {
            opts: ProverOpts::fast(),
            stdout: Vec::new(),
        }
    }

    pub fn prove<T: Serialize>(&mut self, input: T) -> Result<Receipt> {
        let env = ExecutorEnv::builder()
            .write(&input)?
            .stdout(&mut self.stdout)
            .build()?;

        let proof = default_prover().prove_with_opts(env, FISSION_ELF, &self.opts)?;

        Ok(proof.receipt)
    }

    pub fn read<T: DeserializeOwned>(&self) -> Result<T> {
        Ok(risc0_zkvm::serde::from_slice(&self.stdout)?)
    }
}
