mod dleq;

use curve25519_dalek::{
    constants::ED25519_BASEPOINT_POINT as G,
    edwards::EdwardsPoint,
    scalar::Scalar,
};
pub use dleq::*;

use crate::protocol::*;

/// Signs a blinded note value using the mint's secret key
/// B' = blinded note value, secret_key = mint's signing key
/// Returns C' = signed blinded note
pub fn blind_sign(secret_key: SecretKey, b_prime: &EdwardsPoint) -> EdwardsPoint {
    Scalar::from(secret_key) * b_prime
}

/// Blinds a note value before sending to mint for signing
/// note_point = original note point, blinding_factor = random scalar r
/// Returns B' = blinded note value
pub fn blind_note(note_point: &EdwardsPoint, blinding_factor: &Scalar) -> EdwardsPoint {
    note_point + blinding_factor * G
}

/// Unblinds a signed note value to get final signature
/// signed_blinded_note = C', blinding_factor = r, mint_pubkey = A
/// Returns C = unblinded signature
pub fn unblind_signature(
    signed_blinded_note: &EdwardsPoint,
    blinding_factor: &Scalar,
    mint_pubkey: &EdwardsPoint,
) -> EdwardsPoint {
    signed_blinded_note - blinding_factor * mint_pubkey
}

/// Converts a note's fields into a curve point for signing
/// Used before blinding to get initial note point
pub fn note_to_curve_point(note_fields: &[u8]) -> EdwardsPoint {
    let scalar = Scalar::from_bytes_mod_order(todo!());
    scalar * G
}
