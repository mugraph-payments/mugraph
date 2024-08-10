use rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha256};

use crate::types::{Hash, PublicKey, SecretKey};

pub mod dh;
pub mod schnorr;

pub const HTC_SEP: &'static [u8] = b"mugraph_v0_htc";

pub type Point = curve25519_dalek::ristretto::RistrettoPoint;
pub type Scalar = curve25519_dalek::scalar::Scalar;

pub const G: Point = curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;

pub fn hash_to_scalar(data: &[&[u8]]) -> Scalar {
    let mut hasher = Sha256::new();

    for item in data {
        hasher.update(&item);
    }

    let hash: Hash = hasher.finalize().as_slice().try_into().unwrap();
    Scalar::from_bytes_mod_order(*hash)
}

pub fn hash_to_curve(message: &[u8]) -> Point {
    let scalar = hash_to_scalar(&[&*HTC_SEP, message]);
    G * scalar
}

pub fn generate_keypair<R: RngCore + CryptoRng>(rng: &mut R) -> (SecretKey, PublicKey) {
    let secret_key = Scalar::random(rng);
    let pubkey = G * secret_key;

    (secret_key.into(), pubkey.into())
}
