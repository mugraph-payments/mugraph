#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Failed producing proof: {0}")]
    ProofError(String),
}
