mod dleq;

use curve25519_dalek::{
    constants::ED25519_BASEPOINT_POINT as G,
    edwards::EdwardsPoint,
    scalar::Scalar,
};
pub use dleq::*;

use crate::protocol::*;

/// Blinds a note value before sending it to the mint for signing.
///
/// This function applies a blinding factor to the original note point,
/// creating a blinded version that can be sent to the mint for signing
/// without revealing the actual note value.
///
/// # Arguments
///
/// * `note_point` - The original note point to be blinded.
/// * `r` - A random scalar (r) used as the blinding factor.
///
/// # Returns
///
/// Returns B', the blinded note value as an `EdwardsPoint`.
pub fn blind_note(note: &Note, r: &Scalar) -> BlindedValue {
    let point = hash_to_curve(&note.as_bytes());
    (point + r * G).into()
}

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
/// Returns an `EdwardsPoint` representing the hashed note on the curve.
pub fn hash_to_curve(message: &[u8]) -> EdwardsPoint {
    let scalar = Scalar::from_bytes_mod_order(todo!());
    scalar * G
}
