use crate::{crypto::hash_to_scalar, Error, PublicKey, Result, Scalar, SecretKey, Signature, G};
use rand_core::{CryptoRng, RngCore};

pub fn sign<R: RngCore + CryptoRng>(
    rng: &mut R,
    secret_key: &SecretKey,
    message: &[u8],
) -> Signature {
    let k = Scalar::random(rng);
    let r = G * k;
    let e = hash_to_scalar(&[&r.compress().to_bytes(), message]);
    let s = k + e * secret_key;

    Signature { r, s }
}

pub fn verify(public_key: &PublicKey, signature: &Signature, message: &[u8]) -> Result<()> {
    let e = hash_to_scalar(&[&signature.r.compress().to_bytes(), message]);
    let lhs = G * signature.s;
    let rhs = signature.r + public_key * e;

    if lhs == rhs {
        Ok(())
    } else {
        Err(Error::InvalidSignature)
    }
}
