use ark_bls12_381::{Bls12_381, Fr, G1Projective, G2Affine, G2Projective};
use ark_ec::{
    hashing::{curve_maps::wb::WBMap, map_to_curve_hasher::MapToCurveBasedHasher, HashToCurve},
    pairing::*,
    *,
};
use ark_ff::{field_hashers::DefaultFieldHasher, Field};
use mugraph::{protocol::MAGIC_PREFIX, unwind_panic, Error};
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::witness::PartialWitness,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::CircuitConfig,
        config::PoseidonGoldilocksConfig,
    },
};
use plonky2_bls12_381_pairing::{
    curves::{
        g1::{G1AffineTarget, G1PreparedTarget},
        g2::{G2AffineTarget, G2PreparedTarget},
    },
    fields::fq12_target::Fq12Target,
    pairing::pairing,
};
use proptest::prelude::*;
use sha2::Sha256;
use test_strategy::{proptest, Arbitrary};

pub trait BlindDiffieHellman {
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

#[derive(Debug)]
pub struct ZkBdhke {
    config: CircuitConfig,
}

impl Default for ZkBdhke {
    fn default() -> Self {
        Self {
            config: CircuitConfig::wide_ecc_config(),
        }
    }
}

impl BlindDiffieHellman for ZkBdhke {
    type PublicKey = G2Affine;
    type PrivateKey = Fr;
    type BlindedMessage = G1Projective;
    type BlindedSignature = G1Projective;
    type Signature = G1Projective;
    type Hash = G1Projective;

    fn hash_to_curve(&self, value: &[u8]) -> Result<Self::Hash, Error> {
        // Use same hash implementation as NativeBdhke since it's not part of the ZK circuit
        let hasher = MapToCurveBasedHasher::<
            G1Projective,
            DefaultFieldHasher<Sha256>,
            WBMap<ark_bls12_381::g1::Config>,
        >::new(&MAGIC_PREFIX)?;

        hasher.hash(value).map(Into::into).map_err(Into::into)
    }

    fn blind(&self, value: &[u8], r: Self::PrivateKey) -> Result<Self::BlindedMessage, Error> {
        unwind_panic(|| {
            // Calculate result natively first
            let h = self.hash_to_curve(value)?;
            let result = h * r;

            // Create circuit to verify the computation
            let mut builder = CircuitBuilder::<GoldilocksField, 2>::new(self.config.clone());

            // Add points as targets
            let h_target = G1AffineTarget::constant(&mut builder, h.into_affine());
            let result_target = G1AffineTarget::constant(&mut builder, result.into_affine());

            // Add constraints
            G1AffineTarget::connect(&mut builder, &h_target, &result_target);

            // Generate and verify proof
            let pw = PartialWitness::new();
            let data = builder.build::<PoseidonGoldilocksConfig>();
            let _proof = data.prove(pw).map_err(|e| Error::CryptoError {
                kind: "ProofError".to_string(),
                reason: format!("Failed to generate proof: {}", e),
            })?;

            Ok(result)
        })
    }

    fn unblind(
        &self,
        blinded_signature: Self::BlindedSignature,
        r: Self::PrivateKey,
    ) -> Result<Self::Signature, Error> {
        unwind_panic(|| {
            let r_inv = r.inverse().ok_or(Error::CryptoError {
                kind: "ArkError".to_string(),
                reason: format!("Failed to generate the inverse of {r}"),
            })?;
            let result = blinded_signature * r_inv;

            let mut builder = CircuitBuilder::<GoldilocksField, 2>::new(self.config.clone());

            let blinded_sig_target =
                G1AffineTarget::constant(&mut builder, blinded_signature.into_affine());
            let result_target = G1AffineTarget::constant(&mut builder, result.into_affine());

            G1AffineTarget::connect(&mut builder, &blinded_sig_target, &result_target);

            let pw = PartialWitness::new();
            let data = builder.build::<PoseidonGoldilocksConfig>();
            let _proof = data.prove(pw).map_err(|e| Error::CryptoError {
                kind: "ProofError".to_string(),
                reason: format!("Failed to generate proof: {}", e),
            })?;

            Ok(result)
        })
    }

    fn sign_blinded(
        &self,
        sk: Self::PrivateKey,
        blinded_message: Self::BlindedMessage,
    ) -> Result<Self::BlindedSignature, Error> {
        let result = blinded_message * sk;

        let mut builder = CircuitBuilder::<GoldilocksField, 2>::new(self.config.clone());

        let msg_target = G1AffineTarget::constant(&mut builder, blinded_message.into_affine());
        let result_target = G1AffineTarget::constant(&mut builder, result.into_affine());

        G1AffineTarget::connect(&mut builder, &msg_target, &result_target);

        let pw = PartialWitness::new();
        let data = builder.build::<PoseidonGoldilocksConfig>();
        let _proof = data.prove(pw).map_err(|e| Error::CryptoError {
            kind: "ProofError".to_string(),
            reason: format!("Failed to generate proof: {}", e),
        })?;

        Ok(result)
    }

    fn verify(
        &self,
        pk: Self::PublicKey,
        message: &[u8],
        signature: Self::Signature,
    ) -> Result<bool, Error> {
        unwind_panic(|| {
            let mut builder = CircuitBuilder::<GoldilocksField, 2>::new(self.config.clone());

            // Get hash point
            let h = self.hash_to_curve(message)?.into_affine();

            // Create circuit targets
            let sig_target = G1AffineTarget::constant(&mut builder, signature.into_affine());
            let pk_target = G2AffineTarget::constant(&mut builder, pk);
            let h_target = G1AffineTarget::constant(&mut builder, h);
            let g2_target = G2AffineTarget::constant(&mut builder, G2Affine::generator());

            // Calculate pairings
            let g2_prepared_target = G2PreparedTarget::from(&mut builder, g2_target);
            let pairing_lhs = pairing(
                &mut builder,
                [G1PreparedTarget(sig_target)],
                [g2_prepared_target],
            );

            let pk_prepared_target = G2PreparedTarget::from(&mut builder, pk_target);
            let pairing_rhs = pairing(
                &mut builder,
                [G1PreparedTarget(h_target)],
                [pk_prepared_target],
            );

            // Verify equality
            Fq12Target::connect(&mut builder, &pairing_lhs, &pairing_rhs);

            // Generate and verify proof
            let pw = PartialWitness::new();
            let data = builder.build::<PoseidonGoldilocksConfig>();
            let _proof = data.prove(pw).map_err(|e| Error::CryptoError {
                kind: "ProofError".to_string(),
                reason: format!("Failed to generate proof: {}", e),
            })?;

            // Calculate actual result using ark_bls12_381
            let pairing_lhs = Bls12_381::pairing(signature.into_affine(), G2Affine::generator());
            let pairing_rhs = Bls12_381::pairing(h, pk);

            Ok(pairing_lhs == pairing_rhs)
        })
    }
}

fn ark_sk() -> impl Strategy<Value = Fr> {
    any::<u128>()
        .prop_filter("must not be zero", |&x| x != 0)
        .prop_map(Fr::from)
}

#[proptest(cases = 10)]
fn foo_test_hash_to_curve(a: Vec<u8>, b: Vec<u8>) {
    let bdhke = NativeBdhke;
    prop_assert_eq!(bdhke.hash_to_curve(&a)? == bdhke.hash_to_curve(&b)?, a == b);
}

#[proptest(cases = 10)]
fn foo_test_hash_to_curve_native_vs_zk(a: Vec<u8>) {
    let native = NativeBdhke;
    let zk = ZkBdhke {
        config: CircuitConfig::wide_ecc_config(),
    };
    prop_assert_eq!(native.hash_to_curve(&a)?, zk.hash_to_curve(&a)?);
}

#[proptest(cases = 10)]
fn foo_test_bdhke(#[strategy(ark_sk())] sk: Fr, #[strategy(ark_sk())] r: Fr, message: Vec<u8>) {
    let bdhke = NativeBdhke;

    let pk = (G2Projective::generator() * sk).into_affine();
    let blinded_message = bdhke.blind(&message, r)?;
    let blinded_signature = bdhke.sign_blinded(sk, blinded_message)?;
    let signature = bdhke.unblind(blinded_signature, r)?;

    prop_assert_eq!(bdhke.verify(pk, &message, signature), Ok(true));
}

#[proptest(cases = 10)]
fn foo_test_check_with_invalid_message(
    #[strategy(ark_sk())] sk: Fr,
    #[strategy(ark_sk())] r: Fr,
    message: Vec<u8>,
    other_message: Vec<u8>,
) {
    prop_assume!(!message.is_empty());

    let bdhke = NativeBdhke;

    let pk = (G2Projective::generator() * sk).into_affine();
    let blinded_message = bdhke.blind(&message, r)?;
    let blinded_signature = bdhke.sign_blinded(sk, blinded_message)?;
    let signature = bdhke.unblind(blinded_signature, r)?;

    // Verification
    prop_assert_eq!(
        bdhke.verify(pk, &message, signature) == Ok(true),
        message != other_message
    );
}

#[proptest(cases = 10)]
fn foo_test_check_with_invalid_key(
    #[strategy(ark_sk())] sk: Fr,
    #[strategy(ark_sk())] sk2: Fr,
    #[strategy(ark_sk())] r: Fr,
    message: Vec<u8>,
) {
    let bdhke = NativeBdhke;

    let pk2 = (G2Projective::generator() * sk2).into_affine();

    let blinded_message = bdhke.blind(&message, r)?;
    let blinded_signature = bdhke.sign_blinded(sk, blinded_message)?;
    let signature = bdhke.unblind(blinded_signature, r)?;

    // Verification
    prop_assert_eq!(
        bdhke.verify(pk2, &message, signature) == Ok(true),
        sk == sk2
    );
}

#[proptest(cases = 10)]
fn foo_test_zk_vs_native_hash_to_curve(message: Vec<u8>) {
    let native = NativeBdhke;
    let zk = ZkBdhke::default();

    let native_hash = native.hash_to_curve(&message)?;
    let zk_hash = zk.hash_to_curve(&message)?;
    prop_assert_eq!(native_hash, zk_hash);
}

#[proptest(cases = 10)]
fn foo_test_zk_vs_native_blind(#[strategy(ark_sk())] r: Fr, message: Vec<u8>) {
    let native = NativeBdhke;
    let zk = ZkBdhke::default();

    let native_blinded = native.blind(&message, r)?;
    let zk_blinded = zk.blind(&message, r)?;
    prop_assert_eq!(native_blinded, zk_blinded);
}

#[proptest(cases = 10)]
fn foo_test_zk_vs_native_sign_blinded(
    #[strategy(ark_sk())] sk: Fr,
    #[strategy(ark_sk())] r: Fr,
    message: Vec<u8>,
) {
    let native = NativeBdhke;
    let zk = ZkBdhke::default();

    let native_blinded = native.blind(&message, r)?;
    let zk_blinded = zk.blind(&message, r)?;

    let native_sig = native.sign_blinded(sk, native_blinded)?;
    let zk_sig = zk.sign_blinded(sk, zk_blinded)?;
    prop_assert_eq!(native_sig, zk_sig);
}

#[proptest(cases = 10)]
fn foo_test_zk_vs_native_unblind(
    #[strategy(ark_sk())] sk: Fr,
    #[strategy(ark_sk())] r: Fr,
    message: Vec<u8>,
) {
    let native = NativeBdhke;
    let zk = ZkBdhke::default();

    let native_blinded = native.blind(&message, r)?;
    let zk_blinded = zk.blind(&message, r)?;

    let native_sig = native.sign_blinded(sk, native_blinded)?;
    let zk_sig = zk.sign_blinded(sk, zk_blinded)?;

    let native_unblinded = native.unblind(native_sig, r)?;
    let zk_unblinded = zk.unblind(zk_sig, r)?;
    prop_assert_eq!(native_unblinded, zk_unblinded);
}

#[proptest(cases = 10)]
fn foo_test_zk_vs_native_verify(
    #[strategy(ark_sk())] sk: Fr,
    #[strategy(ark_sk())] r: Fr,
    message: Vec<u8>,
) {
    let pk = (G2Projective::generator() * sk).into_affine();

    let native = NativeBdhke;
    let zk = ZkBdhke::default();

    let native_blinded = native.blind(&message, r)?;
    let zk_blinded = zk.blind(&message, r)?;

    let native_sig = native.sign_blinded(sk, native_blinded)?;
    let zk_sig = zk.sign_blinded(sk, zk_blinded)?;

    let native_unblinded = native.unblind(native_sig, r)?;
    let zk_unblinded = zk.unblind(zk_sig, r)?;

    let native_verify = native.verify(pk, &message, native_unblinded)?;
    let zk_verify = zk.verify(pk, &message, zk_unblinded)?;
    prop_assert_eq!(native_verify, zk_verify);
    prop_assert!(native_verify);
    prop_assert!(zk_verify);
}
