use curve25519_dalek::{
    constants::ED25519_BASEPOINT_POINT as G,
    edwards::EdwardsPoint,
    scalar::Scalar,
};
use rand::rngs::OsRng;
use sha2::{Digest, Sha512};

use crate::protocol::{PublicKey, SecretKey};

/// Compute C' = a * B'
pub fn compute_c_prime(secret_key: SecretKey, b_prime: &EdwardsPoint) -> EdwardsPoint {
    let scalar: Scalar = secret_key.into();
    scalar * b_prime
}

/// Generate a DLEQ proof
pub fn generate_dleq_proof(
    secret_key: SecretKey,
    public_key: PublicKey,
    b_prime: &EdwardsPoint,
    c_prime: &EdwardsPoint,
) -> DleqProof {
    let mut rng = OsRng;
    let r = Scalar::random(&mut rng);

    let r1 = &r * &G;
    let r2 = &r * b_prime;

    // Compute e = H(R1 || R2 || A || C')
    let mut hasher = Sha512::new();
    hasher.update(r1.compress().as_bytes());
    hasher.update(r2.compress().as_bytes());
    hasher.update(public_key.into().compress().as_bytes());
    hasher.update(c_prime.compress().as_bytes());
    let e_bytes = hasher.finalize();
    let e = Scalar::from_bytes_mod_order(e_bytes.into());

    let s = r + e * secret_key.into();

    DleqProof { e, s }
}

/// Compute Y = hash_to_curve(secret_message)
pub fn compute_y(secret_message: &[u8]) -> EdwardsPoint {
    hash_to_curve(secret_message)
}

/// Compute B' = Y + r * G
pub fn compute_b_prime(y: &EdwardsPoint, r: &Scalar) -> EdwardsPoint {
    y + r * G
}

/// Compute C = C' - r * A
pub fn compute_c(c_prime: &EdwardsPoint, r: &Scalar, a: &EdwardsPoint) -> EdwardsPoint {
    let c = c_prime - r * a;
    // Ensure `c` is a valid point
    debug_assert!(c.compress().as_bytes().len() == 32, "Invalid point C");
    c
}

/// Verify the DLEQ proof
pub fn verify_dleq_proof(
    proof: &DleqProof,
    b_prime: &EdwardsPoint,
    c_prime: &EdwardsPoint,
    a: &EdwardsPoint,
) -> bool {
    let r1 = &proof.s * G - &proof.e * a;
    let r2 = &proof.s * b_prime - &proof.e * c_prime;

    // Compute e' = H(R1 || R2 || A || C')
    let mut hasher = Sha512::new();
    hasher.update(r1.compress().as_bytes());
    hasher.update(r2.compress().as_bytes());
    hasher.update(a.compress().as_bytes());
    hasher.update(c_prime.compress().as_bytes());
    let e_bytes = hasher.finalize();
    let e_prime = Scalar::from_bytes_mod_order(e_bytes.into());

    proof.e == e_prime
}

/// Hash an arbitrary message to a point on the curve
pub fn hash_to_curve(message: &[u8]) -> EdwardsPoint {
    let mut hasher = Sha512::new();
    hasher.update(message);
    let hash = hasher.finalize();
    let scalar = Scalar::from_bytes_mod_order_wide(&hash.into());
    scalar * G
}

/// DLEQ (Discrete Log Equality) proof
#[derive(Debug)]
pub struct DleqProof {
    pub e: Scalar,
    pub s: Scalar,
}
