use lazy_static::lazy_static;
use rand::rngs::OsRng;

pub mod crypto;
pub mod error;
pub mod types;

#[cfg(test)]
pub mod testing;

pub const DOMAIN_SEPARATOR: &[u8] = b"MUGRAPH_V1_CURVE_25519_HASH_TO_CURVE_";
pub const DLEQ_DOMAIN_SEPARATOR: &[u8] = b"MUGRAPH_V1_CURVE_25519_DLEQ_PROOF_";
pub const COMMITMENT_TRANSCRIPT_LABEL: &[u8] = b"MUGRAPH_V1_CURVE_25519_COMMITMENT_";
pub const COMMITMENT_VERIFIER_LABEL: &[u8] = b"MUGRAPH_V1_CURVE_25519_COMMITMENT_VERIFIER_";
pub const RANGE_PROOF_DOMAIN_SEPARATOR: &[u8] = b"MUGRAPH_V1_CURVE_25519_RANGE_PROOF_";

lazy_static! {
    pub static ref G: types::Point = curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
    pub static ref H: types::Point = types::Point::random(&mut OsRng);
}
