use std::io::ErrorKind;

use onlyerror::Error;
use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::types::{Hash, Signature};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error, Clone, Serialize, Deserialize, Arbitrary, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    #[error("Server error: {reason}")]
    ServerError { reason: String },

    #[error("Simulated error: {reason}")]
    SimulatedError { reason: String },

    #[error("Storage error ({kind}): {reason}")]
    StorageError { kind: String, reason: String },

    #[error("Rng error: {reason}")]
    RngError { reason: String },

    #[error("Insufficient funds for {asset_id}, expected {expected} but got {got}")]
    InsufficientFunds {
        asset_id: Hash,
        expected: u64,
        got: u64,
    },

    #[error("Atom has already been spent: {signature}")]
    AlreadySpent { signature: Signature },

    #[error("Invalid signature {signature}: {reason}")]
    InvalidSignature {
        reason: String,
        signature: Signature,
    },

    #[error("Invalid public or secret key: {reason}")]
    InvalidKey { reason: String },

    #[error("Invalid hash: {reason}")]
    InvalidHash { reason: String },

    #[error("Atom is invalid: {reason}")]
    InvalidAtom { reason: String },

    #[error("Error handling JSON: {reason}")]
    JsonError { reason: String },

    #[error("Unbalanced transaction, expected {pre:?}, got {post:?}")]
    UnbalancedTransaction { pre: Vec<u128>, post: Vec<u128> },

    #[error("Invalid Transaction: {reason}")]
    InvalidTransaction { reason: String },

    #[error("Multiple errors happened at once: {errors:?}")]
    Multiple { errors: Vec<Error> },

    #[error("Other error")]
    Other,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        let reason = e.to_string();
        match e.kind() {
            ErrorKind::Other if reason.contains("injected_error") => {
                Self::SimulatedError { reason }
            }
            k => Self::StorageError {
                kind: k.to_string(),
                reason: e.to_string(),
            },
        }
    }
}

impl From<Error> for std::io::Error {
    fn from(e: Error) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    }
}

impl From<rand::Error> for Error {
    fn from(value: rand::Error) -> Self {
        Error::RngError {
            reason: value.to_string(),
        }
    }
}

#[inline]
fn to_simulated_or_storage_error<T: std::error::Error + ToString>(value: T, kind: &str) -> Error {
    let reason = value.to_string();

    match reason.contains("injected_error") {
        true => Error::SimulatedError { reason },
        false => Error::StorageError {
            kind: kind.to_string(),
            reason,
        },
    }
}

impl From<redb::Error> for Error {
    fn from(value: redb::Error) -> Self {
        to_simulated_or_storage_error(value, "redb::Error")
    }
}

impl From<redb::CommitError> for Error {
    fn from(value: redb::CommitError) -> Self {
        to_simulated_or_storage_error(value, "redb::CommitError")
    }
}

impl From<redb::StorageError> for Error {
    fn from(value: redb::StorageError) -> Self {
        to_simulated_or_storage_error(value, "redb::StorageError")
    }
}

impl From<redb::TableError> for Error {
    fn from(value: redb::TableError) -> Self {
        to_simulated_or_storage_error(value, "redb::TableError")
    }
}

impl From<redb::TransactionError> for Error {
    fn from(value: redb::TransactionError) -> Self {
        to_simulated_or_storage_error(value, "redb::TransactionError")
    }
}

impl From<redb::DatabaseError> for Error {
    fn from(value: redb::DatabaseError) -> Self {
        to_simulated_or_storage_error(value, "redb::DatabaseError")
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonError {
            reason: value.to_string(),
        }
    }
}
