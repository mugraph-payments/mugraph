use blake3::Hasher;
use rand::prelude::{CryptoRng, RngCore};

use crate::{
    error::{Error, Result},
    types::*,
};

pub const HTC_SEP: &[u8] = b"mugraph_v0_htc";

pub type Point = curve25519_dalek::ristretto::RistrettoPoint;
pub type Scalar = curve25519_dalek::scalar::Scalar;

pub const G: Point = curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;

#[derive(Debug, Clone, Copy)]
pub struct BlindedPoint {
    pub factor: Scalar,
    pub point: Point,
}

pub fn blind_note<R: RngCore + CryptoRng>(
    rng: &mut R,
    note: &Note,
) -> BlindedPoint {
    blind(rng, note.commitment().as_ref())
}

pub fn blind<R: RngCore + CryptoRng>(
    rng: &mut R,
    secret_message: &[u8],
) -> BlindedPoint {
    let y = hash_to_curve(secret_message);
    let r = Hash::random(rng).to_scalar();
    let b_prime = y + (G * r);

    BlindedPoint {
        factor: r,
        point: b_prime,
    }
}

pub fn sign_blinded(
    secret_key: &SecretKey,
    blinded_point: &Point,
) -> Blinded<Signature> {
    let res = blinded_point * secret_key.to_scalar();
    Blinded(res.into())
}

pub fn unblind_signature(
    signature: &Blinded<Signature>,
    blinding_factor: &Scalar,
    pubkey: &PublicKey,
) -> Result<Signature> {
    // Validate inputs
    if blinding_factor == &Scalar::ZERO {
        return Err(Error::InvalidBlindingFactor);
    }

    let inner = signature.0;
    let res = inner.to_point()? - (pubkey.to_point()? * blinding_factor);

    Ok(Signature(res.compress().0))
}

pub fn verify(
    public_key: &PublicKey,
    message: &[u8],
    signature: Signature,
) -> Result<bool> {
    let y = hash_to_scalar(&[message]);
    Ok(y * public_key.to_point()? == signature.to_point()?)
}

fn hash_to_scalar(data: &[&[u8]]) -> Scalar {
    let mut hasher = Hasher::new();

    hasher.update(HTC_SEP);

    for d in data {
        hasher.update(&(d.len() as u64).to_le_bytes());
        hasher.update(d);
    }

    Hash(*hasher.finalize().as_bytes()).into()
}

pub fn hash_to_curve(message: &[u8]) -> Point {
    G * hash_to_scalar(&[message])
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rand::prelude::StdRng;
    use test_strategy::proptest;

    use super::*;
    use crate::{testing::rng, types::Keypair};

    #[proptest]
    fn test_hash_to_curve_equality(a: Vec<u8>, b: Vec<u8>) {
        prop_assert_eq!(a == b, hash_to_curve(&a) == hash_to_curve(&b));
    }

    #[proptest]
    fn test_hash_to_scalar_equality(a: Vec<u8>, b: Vec<u8>) {
        prop_assert_eq!(a == b, hash_to_scalar(&[&a]) == hash_to_scalar(&[&b]));
    }

    #[proptest]
    fn test_hash_to_curve_sensitivity(
        #[strategy(any::<Vec<u8>>().prop_filter("must not be empty", |x| !x.is_empty()))]
        a: Vec<u8>,
    ) {
        let mut b = a.clone();
        b[0] = b[0].wrapping_add(1);

        prop_assert_ne!(hash_to_curve(&a), hash_to_curve(&b));
    }

    #[proptest]
    fn test_hash_to_scalar_sensitivity(
        #[strategy(any::<Vec<u8>>().prop_filter("must not be empty", |x| !x.is_empty()))]
        a: Vec<u8>,
    ) {
        let mut b = a.clone();
        b[0] = b[0].wrapping_add(1);

        prop_assert_ne!(hash_to_scalar(&[&a]), hash_to_scalar(&[&b]));
    }

    #[proptest]
    fn test_blinding_workflow(
        #[strategy(rng())] mut rng: StdRng,
        pair: Keypair,
        #[strategy(any::<Vec<u8>>().prop_filter("must not be empty", |x| !x.is_empty()))]
        msg: Vec<u8>,
    ) {
        let blinded = blind(&mut rng, &msg);

        let sig = sign_blinded(&pair.secret_key, &blinded.point);
        let unblinded =
            unblind_signature(&sig, &blinded.factor, &pair.public_key)?;

        prop_assert!(verify(&pair.public_key, &msg, unblinded)?);
    }

    #[proptest]
    fn test_blinding_workflow_tampered_blinding_factor(
        #[strategy(rng())] mut rng: StdRng,
        pair: Keypair,
        #[strategy(any::<Vec<u8>>().prop_filter("must not be empty", |x| !x.is_empty()))]
        msg: Vec<u8>,
    ) {
        let blinded = blind(&mut rng, &msg);
        let signed = sign_blinded(&pair.secret_key, &blinded.point);

        let unblinded = unblind_signature(
            &signed,
            &(blinded.factor + Scalar::ONE),
            &pair.public_key,
        )?;

        prop_assert!(
            !verify(&pair.public_key, &msg, unblinded).unwrap_or(false)
        );
    }

    #[proptest]
    fn test_blinding_workflow_tampered_blinded_point(
        #[strategy(rng())] mut rng: StdRng,
        pair: Keypair,
        #[strategy(any::<Vec<u8>>().prop_filter("must not be empty", |x| !x.is_empty()))]
        msg: Vec<u8>,
    ) {
        let blinded = blind(&mut rng, &msg);
        let signed = sign_blinded(&pair.secret_key, &(blinded.point + G));
        let unblinded =
            unblind_signature(&signed, &blinded.factor, &pair.public_key)?;

        prop_assert!(
            !verify(&pair.public_key, &msg, unblinded).unwrap_or(false)
        );
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
        let unblinded =
            unblind_signature(&sig, &blinded.factor, &pair.public_key)?;

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
        let unblinded =
            unblind_signature(&sig, &blinded.factor, &a.public_key)?;

        prop_assert_eq!(verify(&b.public_key, &msg, unblinded)?, a == b);
    }
}
