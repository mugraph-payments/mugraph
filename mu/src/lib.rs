#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Failed producing proof: {0}")]
    CreateProof(String),

    #[error("Failed verifying proof: {0}")]
    VerifyProof(String),
}
