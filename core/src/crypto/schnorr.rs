use curve25519_dalek::ristretto::CompressedRistretto;
use rand_core::{CryptoRng, RngCore};

use crate::{
    crypto::*,
    error::{Error, Result},
    types::*,
};

pub fn sign<R: RngCore + CryptoRng>(
    rng: &mut R,
    secret_key: &SecretKey,
    message: &[u8],
) -> Signature {
    let k = Scalar::random(rng);

    let r = G * k;
    let r_ = r.compress().to_bytes();

    let e = hash_to_scalar(&[&r_, message]);

    let s = k + e * secret_key.to_scalar();
    let s_ = s.to_bytes();

    Signature {
        r: Hash(r_),
        s: Hash(s_),
    }
}

pub fn verify(public_key: &PublicKey, signature: &Signature, message: &[u8]) -> Result<()> {
    let s = Scalar::from_bytes_mod_order(*signature.s);
    let r = CompressedRistretto::from_slice(&*signature.r)
        .map_err(|_| Error::InvalidSignature)?
        .decompress()
        .ok_or(Error::InvalidSignature)?;

    let e = hash_to_scalar(&[&*signature.r, message]);
    let lhs = G * s;
    let rhs = r + public_key.to_point()? * e;

    if lhs == rhs {
        Ok(())
    } else {
        Err(Error::InvalidSignature)
    }
}

#[cfg(all(test, feature = "proptest"))]
mod tests {
    use proptest::prelude::*;
    use rand::{prelude::*, rngs::StdRng};
    use test_strategy::proptest;

    use crate::crypto::schnorr::*;

    fn rng() -> impl Strategy<Value = StdRng> {
        any::<[u8; 32]>().prop_map(StdRng::from_seed)
    }

    #[proptest]
    fn test_sign_verify(#[strategy(rng())] mut rng: StdRng, pair: Keypair, message: Vec<u8>) {
        let signed = sign(&mut rng, &pair.secret_key, &message);

        prop_assert_eq!(verify(&pair.public_key, &signed, &message), Ok(()));
    }

    #[proptest]
    fn test_verify_wrong_key(
        #[strategy(rng())] mut rng: StdRng,
        a: Keypair,
        b: Keypair,
        message: Vec<u8>,
    ) {
        let signed = sign(&mut rng, &a.secret_key, &message);

        prop_assert_eq!(
            verify(&b.public_key, &signed, &message).is_ok(),
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
        let signed = sign(&mut rng, &pair.secret_key, &message);

        prop_assert_eq!(
            verify(&pair.public_key, &signed, &message2).is_ok(),
            message == message2
        );
    }

    #[proptest]
    fn test_verify_wrong_randomness(
        #[strategy(rng())] mut rng: StdRng,
        pair: Keypair,
        message: Vec<u8>,
    ) {
        let signed = sign(&mut rng, &pair.secret_key, &message);

        let mut signed_ = signed.clone();
        signed_.r[0] = signed_.r[0].wrapping_add(1);

        prop_assert_eq!(
            verify(&pair.public_key, &signed_, &message),
            Err(Error::InvalidSignature)
        );
    }
}
