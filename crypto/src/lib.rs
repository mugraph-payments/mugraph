use mugraph_core::{Hash, HTC_SEP};
use rand_core::{CryptoRng, RngCore};

pub mod dh;
pub mod schnorr;

pub type Point = curve25519_dalek::ristretto::RistrettoPoint;
pub type Scalar = curve25519_dalek::scalar::Scalar;
pub type SecretKey = Scalar;
pub type PublicKey = Point;

pub const G: Point = curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;

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
    let scalar = hash_to_scalar(&[&*HTC_SEP, message]);
    G * scalar
}

pub fn generate_keypair<R: RngCore + CryptoRng>(rng: &mut R) -> (SecretKey, PublicKey) {
    let privkey = Scalar::random(rng);
    let pubkey = G * privkey;
    (privkey, pubkey)
}
