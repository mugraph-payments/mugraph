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
) -> Result<Signature> {
    let k = Scalar::random(rng);

    let r = G * k;
    let r_ = r.compress().to_bytes();

    let e = hash_to_scalar(&[&r_, message]);

    let s = k + e * secret_key.to_scalar()?;
    let s_ = s.to_bytes();

    Ok(Signature {
        r: Hash(r_),
        s: Hash(s_),
    })
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
