use blake3::Hasher;
use rand::prelude::{CryptoRng, RngCore};
use schnorr::SchnorrPair;
use traits::Pair;
use traits::Public;
use traits::Secret;

use crate::{error::Result, types::*};

pub mod schnorr;
pub mod traits;

pub const HTC_SEP: &[u8] = b"mugraph_v0_htc";

pub type Point = curve25519_dalek::ristretto::RistrettoPoint;
pub type Scalar = curve25519_dalek::scalar::Scalar;

pub const G: Point = curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;

pub struct BlindedPoint {
    pub factor: Scalar,
    pub point: Point,
}

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

pub fn sign_blinded<P: Pair>(secret_key: &P::Secret, blinded_point: &Point) -> Blinded<Signature> {
    let res = blinded_point * secret_key.to_scalar();
    Blinded(res.into())
}

pub fn unblind_signature(
    signature: &Blinded<Signature>,
    blinding_factor: &Scalar,
    pubkey: &PublicKey,
) -> Result<Signature> {
    let inner = signature.0;
    let res = inner.to_point()? - (pubkey.to_point()? * blinding_factor);

    Ok(Signature(res.compress().0))
}

pub fn verify<P: Pair>(
    public_key: &P::Public,
    message: &[u8],
    signature: Signature,
) -> Result<bool> {
    let y = hash_to_scalar(&[HTC_SEP, message]);
    Ok(y * public_key.to_point()? == signature.to_point()?)
}

fn hash_to_scalar(data: &[&[u8]]) -> Scalar {
    let mut hasher = Hasher::new();

    for d in data {
        hasher.update(d);
    }

    Hash(*hasher.finalize().as_bytes()).into()
}

pub fn hash_to_curve(message: &[u8]) -> Point {
    let scalar = hash_to_scalar(&[HTC_SEP, message]);
    G * scalar
}

#[cfg(test)]
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

        let sig = sign_blinded::<SchnorrPair>(&pair.secret_key, &blinded.point);
        let unblinded = unblind_signature(&sig, &blinded.factor, &pair.public_key)?;

        prop_assert!(verify::<SchnorrPair>(&pair.public_key, &msg, unblinded)?);
    }

    #[proptest]
    fn test_signature_validity_equality(
        #[strategy(rng())] mut rng: StdRng,
        pair: Keypair,
        a: Vec<u8>,
        b: Vec<u8>,
    ) {
        let blinded = blind(&mut rng, &a);

        let sig = sign_blinded::<SchnorrPair>(&pair.secret_key, &blinded.point);
        let unblinded = unblind_signature(&sig, &blinded.factor, &pair.public_key)?;

        prop_assert_eq!(
            verify::<SchnorrPair>(&pair.public_key, &b, unblinded)?,
            a == b
        );
    }

    #[proptest]
    fn test_signature_key_validity(
        #[strategy(rng())] mut rng: StdRng,
        a: Keypair,
        b: Keypair,
        msg: Vec<u8>,
    ) {
        let blinded = blind(&mut rng, &msg);

        let sig = sign_blinded::<SchnorrPair>(&a.secret_key, &blinded.point);
        let unblinded = unblind_signature(&sig, &blinded.factor, &a.public_key)?;

        prop_assert_eq!(
            verify::<SchnorrPair>(&b.public_key, &msg, unblinded)?,
            a == b
        );
    }
}
