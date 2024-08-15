use curve25519_dalek::digest::*;
use rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha512};

use crate::{error::Result, types::*};

pub const HTC_SEP: &[u8] = b"mugraph_v0_htc";

pub type Point = curve25519_dalek::ristretto::RistrettoPoint;
pub type Scalar = curve25519_dalek::scalar::Scalar;

pub const G: Point = curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;

pub struct BlindedPoint {
    pub factor: Scalar,
    pub point: Point,
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct BlindedSignature(Point);

pub fn blind_note<R: RngCore + CryptoRng>(rng: &mut R, note: &Note) -> BlindedPoint {
    blind(rng, note.commitment().as_ref())
}

pub fn blind<R: RngCore + CryptoRng>(rng: &mut R, secret_message: &[u8]) -> BlindedPoint {
    let y = hash_to_curve(secret_message);
    let r = Scalar::random(rng);
    let b_prime = y + (G * r);

    BlindedPoint {
        factor: r,
        point: b_prime,
    }
}

pub fn sign_blinded(secret_key: &SecretKey, blinded_point: &Point) -> BlindedSignature {
    let res = blinded_point * secret_key.to_scalar();
    BlindedSignature(res)
}

pub fn unblind_signature(
    signature: &BlindedSignature,
    blinding_factor: &Scalar,
    pubkey: &PublicKey,
) -> Result<Signature> {
    let res = signature.0 - (pubkey.to_point()? * blinding_factor);

    Ok(Signature(res.compress().0))
}

pub fn verify(public_key: &PublicKey, message: &[u8], signature: Signature) -> Result<bool> {
    let y = hash_to_scalar(&[HTC_SEP, message]);
    Ok(y * public_key.to_point()? == signature.to_point()?)
}

fn hash_to_scalar(data: &[&[u8]]) -> Scalar {
    let mut hash = Sha512::new();

    for d in data {
        hash = hash.chain(d);
    }

    Scalar::from_hash(hash)
}

fn hash_to_curve(message: &[u8]) -> Point {
    let scalar = hash_to_scalar(&[HTC_SEP, message]);
    G * scalar
}

#[cfg(all(test, feature = "proptest"))]
mod tests {
    use proptest::prelude::*;
    use rand::prelude::StdRng;
    use test_strategy::proptest;

    use super::*;
    use crate::{testing::rng, types::Keypair};

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

    #[proptest]
    fn test_blinding_workflow(#[strategy(rng())] mut rng: StdRng, pair: Keypair, msg: Vec<u8>) {
        let blinded = blind(&mut rng, &msg);

        let sig = sign_blinded(&pair.secret_key, &blinded.point);
        let unblinded = unblind_signature(&sig, &blinded.factor, &pair.public_key)?;

        prop_assert!(verify(&pair.public_key, &msg, unblinded)?);
    }

    #[proptest]
    fn test_signature_validity_equality(
        #[strategy(rng())] mut rng: StdRng,
        pair: Keypair,
        a: Vec<u8>,
        b: Vec<u8>,
    ) {
        let blinded = blind(&mut rng, &a);

        let sig = sign_blinded(&pair.secret_key, &blinded.point);
        let unblinded = unblind_signature(&sig, &blinded.factor, &pair.public_key)?;

        prop_assert_eq!(verify(&pair.public_key, &b, unblinded)?, a == b);
    }

    #[proptest]
    fn test_signature_key_validity(
        #[strategy(rng())] mut rng: StdRng,
        a: Keypair,
        b: Keypair,
        msg: Vec<u8>,
    ) {
        let blinded = blind(&mut rng, &msg);

        let sig = sign_blinded(&a.secret_key, &blinded.point);
        let unblinded = unblind_signature(&sig, &blinded.factor, &a.public_key)?;

        prop_assert_eq!(verify(&b.public_key, &msg, unblinded)?, a == b);
    }
}
