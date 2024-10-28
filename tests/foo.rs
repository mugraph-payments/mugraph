use ark_bls12_381::{Fr, G2Projective};
use ark_ec::*;
use ark_ff::UniformRand;
use proptest::prelude::*;
use rand::rngs::OsRng;
use test_strategy::proptest;

mod native {
    use ark_bls12_381::{g1::Config, Bls12_381, Fr, G1Projective, G2Affine};
    use ark_ec::{
        hashing::{curve_maps::wb::WBMap, map_to_curve_hasher::MapToCurveBasedHasher, HashToCurve},
        pairing::Pairing,
        *,
    };
    use ark_ff::{field_hashers::DefaultFieldHasher, Field};
    use mugraph::{protocol::MAGIC_PREFIX, Error};
    use sha2::Sha256;

    pub fn hash_to_curve(message: &[u8]) -> Result<G1Projective, Error> {
        let hasher =
            MapToCurveBasedHasher::<G1Projective, DefaultFieldHasher<Sha256>, WBMap<Config>>::new(
                &MAGIC_PREFIX,
            )?;

        hasher.hash(message).map(Into::into).map_err(Into::into)
    }

    pub fn blind(message: &[u8], r: &Fr) -> Result<G1Projective, Error> {
        let h = hash_to_curve(message)?;
        Ok(h * r)
    }

    pub fn sign_blinded(sk: &Fr, blinded_message: G1Projective) -> G1Projective {
        blinded_message * sk
    }

    pub fn unblind(blinded_signature: G1Projective, r: &Fr) -> Result<G1Projective, Error> {
        let inv = r.inverse().ok_or(Error::CryptoError {
            kind: "ArkError".to_string(),
            reason: format!("Failed to generate the inverse of {r}"),
        })?;

        Ok(blinded_signature * inv)
    }

    pub fn verify(pk: &G2Affine, message: &[u8], signature: &G1Projective) -> Result<bool, Error> {
        let h = hash_to_curve(message)?.into_affine();
        let pairing_lhs = Bls12_381::pairing(signature.into_affine(), G2Affine::generator());
        let pairing_rhs = Bls12_381::pairing(h, *pk);

        Ok(pairing_lhs == pairing_rhs)
    }
}

#[proptest]
fn foo_test_hash_to_curve(a: Vec<u8>, b: Vec<u8>) {
    use native::*;

    prop_assert_eq!(hash_to_curve(&a)? == hash_to_curve(&b)?, a == b);
}

#[proptest]
fn foo_test_bdhke(message: Vec<u8>) {
    use native::*;

    let sk = Fr::rand(&mut OsRng);
    let pk = (G2Projective::generator() * sk).into_affine();

    // User Blinds the Message
    let r = Fr::rand(&mut OsRng);
    let blinded_message = blind(&message, &r)?;

    // Signer Signs the Blinded Message
    let blinded_signature = sign_blinded(&sk, blinded_message);

    // User Unblinds the Signature
    let signature = unblind(blinded_signature, &r)?;

    // Verification
    prop_assert_eq!(verify(&pk, &message, &signature), Ok(true));
}

#[proptest]
fn foo_test_check_with_invalid_message(message: Vec<u8>, other_message: Vec<u8>) {
    use ark_bls12_381::{Fr, G2Projective};
    use ark_ec::*;
    use ark_ff::UniformRand;
    use native::*;
    use rand::rngs::OsRng;

    let sk = Fr::rand(&mut OsRng);
    let pk = (G2Projective::generator() * sk).into_affine();
    let r = Fr::rand(&mut OsRng);
    let blinded_message = blind(&message, &r)?;
    let blinded_signature = sign_blinded(&sk, blinded_message);
    let signature = unblind(blinded_signature, &r)?;

    // Verification
    prop_assert_eq!(
        verify(&pk, &message, &signature) == Ok(true),
        message != other_message
    );
}
