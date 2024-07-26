use rand::rngs::OsRng;

use super::hash_to_scalar;
use crate::{error::Error, types::*, G};

pub fn sign(private_key: &Scalar, message: &[u8]) -> Signature {
    let k = Scalar::random(&mut OsRng);
    let r = *G * k;
    let e = hash_to_scalar(&[&r.compress().to_bytes(), message]);
    let s = k + e * private_key;

    Signature { r, s }
}

pub fn verify(public_key: &Point, signature: &Signature, message: &[u8]) -> Result<(), Error> {
    let e = hash_to_scalar(&[&signature.r.compress().to_bytes(), message]);
    let lhs = *G * signature.s;
    let rhs = signature.r + public_key * e;

    if lhs == rhs {
        Ok(())
    } else {
        Err(Error::InvalidSignature)
    }
}
