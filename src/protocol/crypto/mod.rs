mod dleq;

use curve25519_dalek::{ristretto::CompressedRistretto, RistrettoPoint};
pub use dleq::*;

use crate::{protocol::*, Error};

/// Converts a note's fields into a curve point for signing.
///
/// This function is used before blinding to get the initial note point.
/// It hashes the input message to a scalar and then multiplies it with
/// the base point to get a point on the curve.
///
/// # Arguments
///
/// * `message` - A byte slice containing the note's fields to be hashed.
///
/// # Returns
///
/// Returns an `RistrettoPoint` representing the hashed note on the curve.
pub fn hash_to_curve(note: &Note) -> Result<RistrettoPoint, Error> {
    let hash: Hash = PoseidonHash::hash_no_pad(&note.as_fields()).into();

    CompressedRistretto::from_slice(&hash.inner())
        .map_err(|e| Error::DecodeError(e.to_string()))?
        .decompress()
        .ok_or(Error::DecodeError("Failed to decompress hash".to_string()))
}
