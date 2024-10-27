pub use curve25519_dalek::{RistrettoPoint as DalekPoint, Scalar as DalekScalar};

use super::EncodeFields;
use crate::Error;

pub const G: DalekPoint = curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;

// pub use self::{point::Point, scalar::Scalar};

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
/// Returns an `DalekPoint` representing the hashed note on the curve.
pub fn hash_to_curve(value: impl EncodeFields) -> Result<DalekPoint, Error> {
    let res: DalekScalar = value.hash().into();
    Ok(res * G)
}
