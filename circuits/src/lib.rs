use mugraph_core::{Error, Result};
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, Receipt};
use serde::{de::DeserializeOwned, Serialize};

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
            .write(&input)
            .map_err(|_| Error::ExecutorWriteValue)?
            .stdout(&mut self.stdout)
            .build()
            .map_err(|_| Error::ExecutorInitialize)?;

        let prover = default_prover();

        Ok(prover
            .prove_with_opts(env, FISSION_ELF, &self.opts)
            .map_err(|_| Error::ProofGenerate)?
            .receipt)
    }

    pub fn read<T: DeserializeOwned>(&self) -> Result<T> {
        risc0_zkvm::serde::from_slice(&self.stdout).map_err(|_| Error::StdoutDecode)
    }
}
