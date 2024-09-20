use curve25519_dalek::ristretto::CompressedRistretto;
use rand::{
    rngs::{StdRng, ThreadRng},
    CryptoRng, RngCore,
};
use traits::{Pair, Seed, Signature};

use crate::{
    crypto::*,
    error::{Error, Result},
};

pub struct SchnorrSignature {
    r: Hash,
    s: Hash,
}

#[derive(Clone, Debug)]
pub struct SchnorrPair {
    public_key: PublicKey,
    secret_key: SecretKey,
}

impl SchnorrPair {
    pub fn new(public_key: PublicKey, secret_key: SecretKey) -> Self {
        SchnorrPair {
            public_key,
            secret_key,
        }
    }
}

impl Signature for SchnorrSignature {}
impl Seed for StdRng {}
impl Seed for ThreadRng {}

impl Pair for SchnorrPair {
    type Signature = SchnorrSignature;
    type Public = PublicKey;
    type Secret = SecretKey;
    type Seed = StdRng;

    fn sign(&self, seed: &mut Self::Seed, message: &[u8]) -> Self::Signature {
        let k = Scalar::random(seed);

        let r = G * k;
        let r_ = r.compress().to_bytes();

        let e = hash_to_scalar(&[&r_, message, self.secret_key.public().to_bytes()]);

        let s = k + e * self.secret_key.to_scalar();
        let s_ = s.to_bytes();

        SchnorrSignature {
            r: Hash(r_),
            s: Hash(s_),
        }
    }

    fn public(&self) -> Self::Public {
        self.public_key
    }

    fn secret(&self) -> Self::Secret {
        self.secret_key
    }

    fn verify(&self, signature: &Self::Signature, message: &[u8]) -> Result<()> {
        let s = Scalar::from_bytes_mod_order(*signature.s);
        let r = CompressedRistretto::from_slice(&*signature.r)
            .map_err(|_| Error::Other)?
            .decompress()
            .ok_or(Error::Other)?;

        let e = hash_to_scalar(&[&*signature.r, message, self.public_key.to_bytes()]);
        let lhs = G * s;
        let rhs = r + self.public_key.to_point()? * e;

        if lhs == rhs {
            Ok(())
        } else {
            Err(Error::Other)
        }
    }

    fn random(seed: &mut impl Seed) -> Self {
        let secret_key = SecretKey::random(seed);
        let public_key = secret_key.public();

        Self {
            public_key,
            secret_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rand::{prelude::*, rngs::StdRng};
    use test_strategy::proptest;

    use crate::crypto::schnorr::*;

    fn rng() -> impl Strategy<Value = StdRng> {
        any::<[u8; 32]>().prop_map(StdRng::from_seed)
    }

    #[proptest]
    fn test_sign_verify(#[strategy(rng())] mut rng: StdRng, pair: Keypair, message: Vec<u8>) {
        let schnorr_pair = SchnorrPair {
            public_key: pair.public_key,
            secret_key: pair.secret_key,
        };
        let signed = schnorr_pair.sign(&mut rng, &message);

        prop_assert_eq!(schnorr_pair.verify(&signed, &message), Ok(()));
    }

    #[proptest]
    fn test_verify_wrong_key(
        #[strategy(rng())] mut rng: StdRng,
        a: Keypair,
        b: Keypair,
        message: Vec<u8>,
    ) {
        let schnorr_pair = SchnorrPair {
            public_key: a.public_key,
            secret_key: a.secret_key,
        };
        let signed = schnorr_pair.sign(&mut rng, &message);

        prop_assert_eq!(
            schnorr_pair.verify(&signed, &message).is_ok(),
            a.public_key == b.public_key
        );
    }

    #[proptest]
    fn test_verify_wrong_message(
        #[strategy(rng())] mut rng: StdRng,
        pair: Keypair,
        message: Vec<u8>,
        message2: Vec<u8>,
    ) {
        let schnorr_pair = SchnorrPair {
            public_key: pair.public_key,
            secret_key: pair.secret_key,
        };
        let signed = schnorr_pair.sign(&mut rng, &message);

        prop_assert_eq!(
            schnorr_pair.verify(&signed, &message2).is_ok(),
            message == message2
        );
    }

    #[proptest]
    fn test_verify_wrong_randomness(
        #[strategy(rng())] mut rng: StdRng,
        pair: Keypair,
        message: Vec<u8>,
    ) {
        let schnorr_pair = SchnorrPair {
            public_key: pair.public_key,
            secret_key: pair.secret_key,
        };
        let signed = schnorr_pair.sign(&mut rng, &message);
        let mut signed_ = signed;
        signed_.r[0] = signed_.r[0].wrapping_add(1);

        prop_assert_eq!(schnorr_pair.verify(&signed_, &message), Err(Error::Other));
    }

    #[proptest]
    fn test_verify_rogue_key(#[strategy(rng())] mut rng: StdRng, pair: Keypair, message: Vec<u8>) {
        let schnorr_pair = SchnorrPair {
            public_key: pair.public_key,
            secret_key: pair.secret_key,
        };
        let signed = schnorr_pair.sign(&mut rng, &message);

        // Create a rogue public key
        let rogue_scalar = Scalar::random(&mut rng);
        let pubkey_point: Point = pair.public_key.to_point()?;
        let rogue_public_key = pubkey_point + G * rogue_scalar;
        let rogue_schnorr_pair = SchnorrPair {
            public_key: rogue_public_key.into(),
            secret_key: pair.secret_key,
        };

        // The signature should not verify with the rogue key
        prop_assert_eq!(
            rogue_schnorr_pair.verify(&signed, &message),
            Err(Error::Other)
        );
    }
}
