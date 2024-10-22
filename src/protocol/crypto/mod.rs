mod dleq;

use curve25519_dalek::{
    constants::ED25519_BASEPOINT_POINT as G,
    edwards::EdwardsPoint,
    scalar::Scalar,
};
pub use dleq::*;

use crate::protocol::*;

/// Compute C' = a * B'
pub fn compute_c_prime(secret_key: SecretKey, b_prime: &EdwardsPoint) -> EdwardsPoint {
    Scalar::from(secret_key) * b_prime
}

/// Compute B' = Y + r * G
pub fn compute_b_prime(y: &EdwardsPoint, r: &Scalar) -> EdwardsPoint {
    y + r * G
}

/// Compute C = C' - r * A
pub fn compute_c(c_prime: &EdwardsPoint, r: &Scalar, a: &EdwardsPoint) -> EdwardsPoint {
    let c = c_prime - r * a;
    c
}

/// Hash an arbitrary message to a point on the curve
pub fn hash_to_curve(message: &[u8]) -> EdwardsPoint {
    let scalar = Scalar::from_bytes_mod_order(todo!());
    scalar * G
}
