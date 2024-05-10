pub mod event;
pub mod proof;
pub mod types;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Failed producing proof: {0}")]
    ProofError(String),
}
