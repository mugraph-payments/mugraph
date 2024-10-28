use onlyerror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    #[error("Transaction is unbalanced, expected {expected} {asset_name}, got {got}")]
    UnbalancedTransaction {
        asset_name: String,
        expected: u64,
        got: u128,
    },

    #[error("Storage Error: {kind} - {reason}")]
    StorageError { kind: String, reason: String },

    #[error("Crypto Error: {reason}")]
    CryptoError { kind: String, reason: String },

    #[error("Mint is improperly configured: {reason}")]
    MintConfiguration { reason: String },

    #[error("Invalid hostname: {0}")]
    InvalidHostname(String),

    #[error("DNS Error: {0}")]
    DNSError(String),

    #[error("Network Error: {0}")]
    NetworkError(String),

    #[error("Decode Error: {0}")]
    DecodeError(String),

    #[error("Panic: {0}")]
    Panic(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::StorageError {
            kind: err.kind().to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<redb::DatabaseError> for Error {
    fn from(err: redb::DatabaseError) -> Self {
        Error::StorageError {
            kind: "DatabaseError".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Error::InvalidHostname(err.to_string())
    }
}

impl From<hickory_resolver::error::ResolveError> for Error {
    fn from(err: hickory_resolver::error::ResolveError) -> Self {
        Error::InvalidHostname(err.to_string())
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::CryptoError {
            kind: err.root_cause().to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<ark_ec::hashing::HashToCurveError> for Error {
    fn from(err: ark_ec::hashing::HashToCurveError) -> Self {
        use std::error::Error as _;

        Error::CryptoError {
            kind: err
                .source()
                .map(|x| x.to_string())
                .unwrap_or("HashToCurveError".to_string()),
            reason: err.to_string(),
        }
    }
}
