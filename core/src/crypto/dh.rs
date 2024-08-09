use rand_core::{CryptoRng, RngCore};

use crate::{
    crypto::*,
    error::{Error, Result},
};

pub fn blind<R: RngCore + CryptoRng>(rng: &mut R, secret_message: &[u8]) -> (Point, Scalar, Point) {
    let y = hash_to_curve(secret_message);
    let r = Scalar::random(rng);
    let b_prime = y + (G * r);
    (y, r, b_prime)
}

pub fn sign_blinded(secret_key: &SecretKey, blinded_point: &Point) -> Point {
    blinded_point * secret_key
}

pub fn unblind_signature(signed_point: &Point, blinding_factor: &Scalar) -> Point {
    signed_point - (G * blinding_factor)
}

pub fn verify_unblinded_point(
    secret_key: &SecretKey,
    message: &[u8],
    unblinded_point: &Point,
) -> Result<()> {
    if hash_to_curve(message) * secret_key == *unblinded_point {
        Ok(())
    } else {
        Err(Error::InvalidUnblindedPoint)
    }
}
