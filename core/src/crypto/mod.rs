use curve25519_dalek::digest::*;
use sha2::{Digest, Sha512};

pub mod schnorr;

pub const HTC_SEP: &[u8] = b"mugraph_v0_htc";

pub type Point = curve25519_dalek::ristretto::RistrettoPoint;
pub type Scalar = curve25519_dalek::scalar::Scalar;

pub const G: Point = curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;

pub fn hash_to_scalar(data: &[&[u8]]) -> Scalar {
    let mut hash = Sha512::new();

    for d in data {
        hash = hash.chain(d);
    }

    Scalar::from_hash(hash)
}

pub fn hash_to_curve(message: &[u8]) -> Point {
    let scalar = hash_to_scalar(&[HTC_SEP, message]);
    G * scalar
}

#[cfg(all(test, feature = "proptest"))]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn test_hash_to_curve(a: Vec<u8>, b: Vec<u8>) {
        prop_assert_eq!(
            a == b,
            hash_to_curve(a.as_ref()) == hash_to_curve(b.as_ref())
        )
    }

    #[proptest]
    fn test_hash_to_scalar(a: Vec<u8>, b: Vec<u8>) {
        prop_assert_eq!(
            a == b,
            hash_to_scalar(&[a.as_ref()]) == hash_to_scalar(&[b.as_ref()])
        )
    }
}
