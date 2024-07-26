pub mod dh;
pub mod proof;
pub mod schnorr;

use blake2::{Blake2b, Digest};
use rand::rngs::OsRng;

use crate::{types::*, DOMAIN_SEPARATOR, G};

pub fn hash_to_scalar(data: &[&[u8]]) -> Scalar {
    let mut hasher = Blake2b::new();

    for item in data {
        hasher.update(item);
    }

    Scalar::from_bytes_mod_order(hasher.finalize().into())
}

pub fn hash_to_curve(message: &[u8]) -> Point {
    let scalar = hash_to_scalar(&[DOMAIN_SEPARATOR, message]);
    *G * scalar
}

pub fn generate_keypair() -> (SecretKey, PublicKey) {
    let privkey = Scalar::random(&mut OsRng);
    let pubkey = *G * privkey;
    (privkey, pubkey)
}
