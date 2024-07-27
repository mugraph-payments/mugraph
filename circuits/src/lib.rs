mod error;

pub mod swap {
    pub use mugraph_circuits_swap::{ELF, ID};
}

pub use self::error::{Error, Result};
pub use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};