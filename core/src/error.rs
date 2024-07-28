use onlyerror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid Hash")]
    InvalidHash,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid unblinded point")]
    InvalidUnblindedPoint,
}
