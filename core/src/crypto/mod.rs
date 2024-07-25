pub mod commitment;
pub mod dh;
pub mod schnorr;

use blake2::{Blake2b, Digest};
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use lazy_static::lazy_static;
use rand::rngs::OsRng;

pub use bulletproofs::RangeProof;
pub use curve25519_dalek::{
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
    traits::*,
};

pub use schnorr::Signature;

pub const DOMAIN_SEPARATOR: &[u8] = b"MUGRAPH_V1_CURVE_25519_HASH_TO_CURVE_";
pub const DLEQ_DOMAIN_SEPARATOR: &[u8] = b"MUGRAPH_V1_CURVE_25519_DLEQ_PROOF_";
pub const COMMITMENT_TRANSCRIPT_LABEL: &[u8] = b"MUGRAPH_V1_CURVE_25519_COMMITMENT_";
pub const COMMITMENT_VERIFIER_LABEL: &[u8] = b"MUGRAPH_V1_CURVE_25519_COMMITMENT_VERIFIER_";
pub const RANGE_PROOF_DOMAIN_SEPARATOR: &[u8] = b"MUGRAPH_V1_CURVE_25519_RANGE_PROOF_";

pub type PublicKey = RistrettoPoint;
pub type SecretKey = Scalar;

lazy_static! {
    pub static ref G: RistrettoPoint = RISTRETTO_BASEPOINT_POINT;
    pub static ref H: RistrettoPoint = RistrettoPoint::random(&mut OsRng);
}

pub fn hash_to_scalar(data: &[&[u8]]) -> Scalar {
    let mut hasher = Blake2b::new();

    for item in data {
        hasher.update(item);
    }

    Scalar::from_bytes_mod_order(hasher.finalize().into())
}

pub fn hash_to_curve(message: &[u8]) -> RistrettoPoint {
    let scalar = hash_to_scalar(&[DOMAIN_SEPARATOR, message]);
    RISTRETTO_BASEPOINT_POINT * scalar
}

pub fn generate_keypair() -> (SecretKey, PublicKey) {
    let privkey = Scalar::random(&mut OsRng);
    let pubkey = RISTRETTO_BASEPOINT_POINT * privkey;
    (privkey, pubkey)
}
