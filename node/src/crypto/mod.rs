pub mod diffie_hellman;
pub mod schnorr;

use blake2::{Blake2b, Digest};
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use rand::rngs::OsRng;

pub use curve25519_dalek::{ristretto::RistrettoPoint, scalar::Scalar};

pub const DOMAIN_SEPARATOR: &[u8] = b"MUGRAPH_V1_CURVE_25519_HASH_TO_CURVE_";
pub const DLEQ_DOMAIN_SEPARATOR: &[u8] = b"MUGRAPH_V1_CURVE_25519_DLEQ_PROOF_";

pub fn hash_to_scalar(data: &[&[u8]]) -> Scalar {
    let mut hasher = Blake2b::new();
    for item in data {
        hasher.update(item);
    }
    Scalar::from_bytes_mod_order_wide(&hasher.finalize().try_into().unwrap())
}

pub fn hash_to_curve(message: &[u8]) -> RistrettoPoint {
    let scalar = hash_to_scalar(&[DOMAIN_SEPARATOR, message]);
    RISTRETTO_BASEPOINT_POINT * scalar
}

pub fn generate_keypair() -> (Scalar, RistrettoPoint) {
    let privkey = Scalar::random(&mut OsRng);
    let pubkey = RISTRETTO_BASEPOINT_POINT * privkey;
    (privkey, pubkey)
}
