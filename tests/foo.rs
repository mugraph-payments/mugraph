use ark_bls12_381::{Bls12_381, Fr, G1Projective, G2Affine, G2Projective};
use ark_ec::{
    hashing::{curve_maps::wb::WBMap, map_to_curve_hasher::MapToCurveBasedHasher, HashToCurve},
    pairing::*,
    *,
};
use ark_ff::{field_hashers::DefaultFieldHasher, Field, UniformRand};
use mugraph::{protocol::MAGIC_PREFIX, Error};
use proptest::prelude::*;
use rand::rngs::OsRng;
use sha2::Sha256;
use test_strategy::{proptest, Arbitrary};

pub trait BlindDiffieHellman: Arbitrary {
    type PublicKey;
    type PrivateKey;
    type BlindedMessage;
    type BlindedSignature;
    type Signature;
    type Hash;

    fn hash_to_curve(&self, value: &[u8]) -> Result<Self::Hash, Error>;
    fn blind(&self, value: &[u8], r: Self::PrivateKey) -> Result<Self::BlindedMessage, Error>;
    fn unblind(
        &self,
        blinded_signature: Self::BlindedSignature,
        r: Self::PrivateKey,
    ) -> Result<Self::Signature, Error>;
    fn sign_blinded(
        &self,
        sk: Self::PrivateKey,
        blinded_message: Self::BlindedMessage,
    ) -> Result<Self::BlindedSignature, Error>;
    fn verify(
        &self,
        pk: Self::PublicKey,
        message: &[u8],
        signature: Self::Signature,
    ) -> Result<bool, Error>;
}

#[derive(Debug, Arbitrary)]
pub struct NativeBdhke;

impl BlindDiffieHellman for NativeBdhke {
    type PublicKey = G2Affine;
    type PrivateKey = Fr;
    type BlindedMessage = G1Projective;
    type BlindedSignature = G1Projective;
    type Signature = G1Projective;
    type Hash = G1Projective;

    fn hash_to_curve(&self, value: &[u8]) -> Result<Self::Hash, Error> {
        let hasher = MapToCurveBasedHasher::<
            G1Projective,
            DefaultFieldHasher<Sha256>,
            WBMap<ark_bls12_381::g1::Config>,
        >::new(&MAGIC_PREFIX)?;

        hasher.hash(value).map(Into::into).map_err(Into::into)
    }

    fn blind(&self, value: &[u8], r: Self::PrivateKey) -> Result<Self::BlindedMessage, Error> {
        let r: &Fr = &r;
        let h = self.hash_to_curve(value)?;
        Ok(h * r)
    }

    fn unblind(
        &self,
        blinded_signature: Self::BlindedSignature,
        r: Self::PrivateKey,
    ) -> Result<Self::Signature, Error> {
        let r: &Fr = &r;
        let inv = r.inverse().ok_or(Error::CryptoError {
            kind: "ArkError".to_string(),
            reason: format!("Failed to generate the inverse of {r}"),
        })?;

        Ok(blinded_signature * inv)
    }

    fn sign_blinded(
        &self,
        sk: Self::PrivateKey,
        blinded_message: Self::BlindedMessage,
    ) -> Result<Self::BlindedSignature, Error> {
        Ok(blinded_message * sk)
    }

    fn verify(
        &self,
        pk: Self::PublicKey,
        message: &[u8],
        signature: Self::Signature,
    ) -> Result<bool, Error> {
        let pk: &G2Affine = &pk;
        let signature: &G1Projective = &signature;
        let h = self.hash_to_curve(message)?.into_affine();
        let pairing_lhs = Bls12_381::pairing(signature.into_affine(), G2Affine::generator());
        let pairing_rhs = Bls12_381::pairing(h, *pk);

        Ok(pairing_lhs == pairing_rhs)
    }
}

fn ark_sk() -> impl Strategy<Value = Fr> {
    any::<u128>().prop_map(Fr::from)
}

#[proptest]
fn foo_test_hash_to_curve(a: Vec<u8>, b: Vec<u8>) {
    let bdhke = NativeBdhke;
    prop_assert_eq!(bdhke.hash_to_curve(&a)? == bdhke.hash_to_curve(&b)?, a == b);
}

#[proptest]
fn foo_test_bdhke(#[strategy(ark_sk())] sk: Fr, message: Vec<u8>) {
    let bdhke = NativeBdhke;

    let pk = (G2Projective::generator() * sk).into_affine();
    let r = Fr::rand(&mut OsRng);
    let blinded_message = bdhke.blind(&message, r)?;
    let blinded_signature = bdhke.sign_blinded(sk, blinded_message)?;
    let signature = bdhke.unblind(blinded_signature, r)?;

    prop_assert_eq!(bdhke.verify(pk, &message, signature), Ok(true));
}

#[proptest]
fn foo_test_check_with_invalid_message(message: Vec<u8>, other_message: Vec<u8>) {
    prop_assume!(!message.is_empty());

    let bdhke = NativeBdhke;

    let sk = Fr::rand(&mut OsRng);
    let pk = (G2Projective::generator() * sk).into_affine();
    let r = Fr::rand(&mut OsRng);
    let blinded_message = bdhke.blind(&message, r)?;
    let blinded_signature = bdhke.sign_blinded(sk, blinded_message)?;
    let signature = bdhke.unblind(blinded_signature, r)?;

    // Verification
    prop_assert_eq!(
        bdhke.verify(pk, &message, signature) == Ok(true),
        message != other_message
    );
}

#[proptest]
fn foo_test_check_with_invalid_key(
    #[strategy(ark_sk())] sk: Fr,
    #[strategy(ark_sk())] sk2: Fr,
    message: Vec<u8>,
) {
    let bdhke = NativeBdhke;

    let pk2 = (G2Projective::generator() * sk2).into_affine();

    let r = Fr::rand(&mut OsRng);
    let blinded_message = bdhke.blind(&message, r)?;
    let blinded_signature = bdhke.sign_blinded(sk, blinded_message)?;
    let signature = bdhke.unblind(blinded_signature, r)?;

    // Verification
    prop_assert_eq!(
        bdhke.verify(pk2, &message, signature) == Ok(true),
        sk == sk2
    );
}
