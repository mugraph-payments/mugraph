use rand_core::{CryptoRng, RngCore};

pub mod dh;
pub mod schnorr;

use crate::{Hash, Point, PublicKey, Scalar, SecretKey, DOMAIN_SEPARATOR, G};

pub fn hash_to_scalar(data: &[&[u8]]) -> Scalar {
    let mut hash = Hash::default();

    for item in data {
        if hash.is_empty() {
            hash = Hash::digest(item).unwrap();
        } else {
            hash = Hash::combine(hash, Hash::digest(item).unwrap()).unwrap();
        }
    }

    Scalar::from_bytes_mod_order(*hash)
}

pub fn hash_to_curve(message: &[u8]) -> Point {
    let scalar = hash_to_scalar(&[DOMAIN_SEPARATOR, message]);
    G * scalar
}

pub fn generate_keypair<R: RngCore + CryptoRng>(rng: &mut R) -> (SecretKey, PublicKey) {
    let privkey = Scalar::random(rng);
    let pubkey = G * privkey;
    (privkey, pubkey)
}
