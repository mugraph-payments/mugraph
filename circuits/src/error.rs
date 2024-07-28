use onlyerror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Prover error: {0}")]
    Prover(String),

    #[error("Mugraph error: {0}")]
    Core(#[from] mugraph_core::Error),

    #[error("Decode error: {0}")]
    Decode(#[from] risc0_zkvm::serde::Error),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Prover(err.to_string())
    }
}
