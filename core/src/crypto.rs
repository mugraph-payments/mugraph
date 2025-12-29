use blake3::Hasher;
use rand::prelude::{CryptoRng, RngCore};

use crate::{
    error::{Error, Result},
    types::*,
};

pub const HTC_SEP: &[u8] = b"mugraph_v0_htc";
pub const DLEQ_SEP: &[u8] = b"mugraph_v0_dleq";

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

pub fn sign_blinded<R: RngCore + CryptoRng>(
    rng: &mut R,
    secret_key: &SecretKey,
    blinded_point: &Point,
) -> BlindSignature {
    let signed_point = blinded_point * secret_key.to_scalar();
    let proof = prove_dleq(rng, secret_key, blinded_point, &signed_point);

    BlindSignature {
        signature: Blinded(signed_point.into()),
        proof,
    }
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

pub fn prove_dleq<R: RngCore + CryptoRng>(
    rng: &mut R,
    secret_key: &SecretKey,
    blinded_point: &Point,
    signed_point: &Point,
) -> DleqProof {
    let k = Hash::random(rng).to_scalar();
    let r_g = G * k;
    let r_b = blinded_point * k;

    let public_key = secret_key.public();
    let challenge =
        dleq_challenge(blinded_point, signed_point, &public_key, &r_g, &r_b);
    let response = k + challenge * secret_key.to_scalar();

    DleqProof {
        challenge: challenge.into(),
        response: response.into(),
    }
}

pub fn verify_dleq(
    public_key: &PublicKey,
    blinded_point: &Point,
    signed_point: &Point,
    proof: &DleqProof,
) -> Result<bool> {
    let e = proof.challenge.to_scalar();
    let z = proof.response.to_scalar();

    let r_g = (G * z) - (public_key.to_point()? * e);
    let r_b = (blinded_point * z) - (signed_point * e);

    let expected =
        dleq_challenge(blinded_point, signed_point, public_key, &r_g, &r_b);

    Ok(e == expected)
}

pub fn verify_dleq_signature(
    public_key: &PublicKey,
    blinded_point: &Point,
    signature: &Blinded<Signature>,
    proof: &DleqProof,
) -> Result<bool> {
    verify_dleq(public_key, blinded_point, &signature.0.to_point()?, proof)
}

fn dleq_challenge(
    blinded_point: &Point,
    signed_point: &Point,
    public_key: &PublicKey,
    r_g: &Point,
    r_b: &Point,
) -> Scalar {
    hash_to_scalar_with_domain(
        DLEQ_SEP,
        &[
            G.compress().as_bytes(),
            blinded_point.compress().as_bytes(),
            public_key.as_ref(),
            signed_point.compress().as_bytes(),
            r_g.compress().as_bytes(),
            r_b.compress().as_bytes(),
        ],
    )
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
    hash_to_scalar_with_domain(HTC_SEP, data)
}

fn hash_to_scalar_with_domain(domain: &[u8], data: &[&[u8]]) -> Scalar {
    let mut hasher = Hasher::new();

    hasher.update(domain);

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

        let sig = sign_blinded(&mut rng, &pair.secret_key, &blinded.point);
        prop_assert!(verify_dleq_signature(
            &pair.public_key,
            &blinded.point,
            &sig.signature,
            &sig.proof
        )?);

        let unblinded = unblind_signature(
            &sig.signature,
            &blinded.factor,
            &pair.public_key,
        )?;

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
        let signed = sign_blinded(&mut rng, &pair.secret_key, &blinded.point);

        let unblinded = unblind_signature(
            &signed.signature,
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
        let signed =
            sign_blinded(&mut rng, &pair.secret_key, &(blinded.point + G));
        let unblinded = unblind_signature(
            &signed.signature,
            &blinded.factor,
            &pair.public_key,
        )?;

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

        let sig = sign_blinded(&mut rng, &pair.secret_key, &blinded.point);
        let unblinded = unblind_signature(
            &sig.signature,
            &blinded.factor,
            &pair.public_key,
        )?;

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

        let sig = sign_blinded(&mut rng, &a.secret_key, &blinded.point);
        let unblinded =
            unblind_signature(&sig.signature, &blinded.factor, &a.public_key)?;

        prop_assert_eq!(verify(&b.public_key, &msg, unblinded)?, a == b);
    }

    #[proptest]
    fn test_dleq_proof_tampering_detected(
        #[strategy(rng())] mut rng: StdRng,
        pair: Keypair,
        #[strategy(any::<Vec<u8>>().prop_filter("must not be empty", |x| !x.is_empty()))]
        msg: Vec<u8>,
    ) {
        let blinded = blind(&mut rng, &msg);
        let sig = sign_blinded(&mut rng, &pair.secret_key, &blinded.point);

        let mut bad_proof = sig.proof;
        bad_proof.response = Hash::random(&mut rng);

        prop_assert!(!verify_dleq_signature(
            &pair.public_key,
            &blinded.point,
            &sig.signature,
            &bad_proof
        )?);
    }
}
