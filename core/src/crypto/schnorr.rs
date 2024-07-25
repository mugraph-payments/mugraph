use curve25519_dalek::{constants::RISTRETTO_BASEPOINT_POINT, RistrettoPoint, Scalar};
use rand::rngs::OsRng;

use super::hash_to_scalar;
use crate::error::Error;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(test_strategy::Arbitrary))]
pub struct Signature {
    #[cfg_attr(test, strategy(crate::testing::point()))]
    pub r: RistrettoPoint,
    #[cfg_attr(test, strategy(crate::testing::scalar()))]
    pub s: Scalar,
}

pub fn sign(private_key: &Scalar, message: &[u8]) -> Signature {
    let k = Scalar::random(&mut OsRng);
    let r = RISTRETTO_BASEPOINT_POINT * k;
    let e = hash_to_scalar(&[&r.compress().to_bytes(), message]);
    let s = k + e * private_key;

    Signature { r, s }
}

pub fn verify(
    public_key: &RistrettoPoint,
    signature: &Signature,
    message: &[u8],
) -> Result<(), Error> {
    let e = hash_to_scalar(&[&signature.r.compress().to_bytes(), message]);
    let lhs = RISTRETTO_BASEPOINT_POINT * signature.s;
    let rhs = signature.r + public_key * e;

    if lhs == rhs {
        Ok(())
    } else {
        Err(Error::InvalidSignature)
    }
}
