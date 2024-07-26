use rand::rngs::OsRng;

use super::*;
use crate::{error::Error, types::*, G};

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(test_strategy::Arbitrary))]
pub struct DLEQProof {
    #[cfg_attr(test, strategy(crate::testing::scalar()))]
    e: Scalar,
    #[cfg_attr(test, strategy(crate::testing::scalar()))]
    s: Scalar,
}

pub fn blind(secret_message: &[u8]) -> (Point, Scalar, Point) {
    let y = hash_to_curve(secret_message);
    let r = Scalar::random(&mut OsRng);
    let b_prime = y + (*G * r);
    (y, r, b_prime)
}

pub fn sign_blinded(secret_key: &SecretKey, blinded_point: &Point) -> (Point, DLEQProof) {
    let signed_point = blinded_point * secret_key;
    let public_key = *G * secret_key;

    // Generate DLEQ proof
    let r = Scalar::random(&mut OsRng);
    let r1 = *G * r;
    let r2 = blinded_point * r;
    let e = hash_to_scalar(&[
        r1.compress().as_bytes(),
        r2.compress().as_bytes(),
        public_key.compress().as_bytes(),
        signed_point.compress().as_bytes(),
    ]);
    let s = r + e * secret_key;

    (signed_point, DLEQProof { e, s })
}

pub fn verify_dleq_proof(
    public_key: &Point,
    blinded_point: &Point,
    signed_point: &Point,
    proof: &DLEQProof,
) -> Result<(), Error> {
    let r1 = (*G * proof.s) - (public_key * proof.e);
    let r2 = (blinded_point * proof.s) - (signed_point * proof.e);
    let e = hash_to_scalar(&[
        r1.compress().as_bytes(),
        r2.compress().as_bytes(),
        public_key.compress().as_bytes(),
        signed_point.compress().as_bytes(),
    ]);

    if e == proof.e {
        Ok(())
    } else {
        Err(Error::InvalidDLEQProof)
    }
}

pub fn unblind_and_verify_signature(
    signed_point: &Point,
    blinding_factor: &Scalar,
    public_key: &Point,
    proof: &DLEQProof,
    blinded_point: &Point,
) -> Result<Point, Error> {
    verify_dleq_proof(public_key, blinded_point, signed_point, proof)?;

    Ok(signed_point - (public_key * blinding_factor))
}

pub fn verify_unblinded_point(
    secret_key: &SecretKey,
    message: &[u8],
    unblinded_point: &Point,
) -> Result<(), Error> {
    if hash_to_curve(message) * secret_key == *unblinded_point {
        Ok(())
    } else {
        Err(Error::InvalidUnblindedPoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::*;
    use test_strategy::proptest;

    #[proptest]
    fn test_blind_diffie_hellman_flow(
        #[strategy(keypair())] a: (Scalar, Point),
        secret_message: Vec<u8>,
    ) {
        // Alice initializes
        let (a, a_pub) = a;

        // Bob blinds the secret message
        let (_, r, b_prime) = blind(&secret_message);

        // Alice signs and produces Schnorr signature
        let (c_prime, proof) = sign_blinded(&a, &b_prime);

        // Bob unblinds and verifies Schnorr signature
        let c = unblind_and_verify_signature(&c_prime, &r, &a_pub, &proof, &b_prime)?;

        // Alice verifies the unblinded signature
        verify_unblinded_point(&a, &secret_message, &c)?;
    }

    #[proptest]
    #[should_panic]
    fn test_schnorr_signature_tampering(
        #[strategy(keypair())] a: (Scalar, Point),
        secret_message: Vec<u8>,
    ) {
        // Alice initializes
        let (a, a_pub) = a;

        // Bob blinds the secret message
        let (_, r, b_prime) = blind(&secret_message);

        // Alice signs and produces Schnorr signature
        let (c_prime, proof) = sign_blinded(&a, &b_prime);

        // Tamper with the signature
        let tampered_proof = DLEQProof {
            e: proof.e + Scalar::ONE,
            s: proof.s,
        };

        // Bob tries to unblind with tampered signature
        unblind_and_verify_signature(&c_prime, &r, &a_pub, &tampered_proof, &b_prime)?;
    }
}
