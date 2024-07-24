use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use rand::rngs::OsRng;

use super::schnorr::{sign, verify, Signature};
use super::*;

pub use curve25519_dalek::{ristretto::RistrettoPoint, scalar::Scalar};

pub fn blind(secret_message: &[u8]) -> (RistrettoPoint, Scalar, RistrettoPoint) {
    let y = hash_to_curve(secret_message);
    let r = Scalar::random(&mut OsRng);
    let b_prime = y + (RISTRETTO_BASEPOINT_POINT * r);
    (y, r, b_prime)
}

pub fn sign_blinded(
    private_key: &Scalar,
    blinded_point: &RistrettoPoint,
) -> (RistrettoPoint, Signature) {
    let signed_point = blinded_point * private_key;
    let signature = sign(private_key, &signed_point.compress().to_bytes());
    (signed_point, signature)
}

pub fn unblind_and_verify_signature(
    signed_point: &RistrettoPoint,
    blinding_factor: &Scalar,
    public_key: &RistrettoPoint,
    signature: &Signature,
) -> Option<RistrettoPoint> {
    if verify(public_key, signature, &signed_point.compress().to_bytes()) {
        Some(signed_point - (public_key * blinding_factor))
    } else {
        None
    }
}

pub fn verify_unblinded_point(
    private_key: &Scalar,
    message: &[u8],
    unblinded_point: &RistrettoPoint,
) -> bool {
    let y = hash_to_curve(message);
    &y * private_key == *unblinded_point
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use test_strategy::proptest;

    fn scalar() -> impl Strategy<Value = Scalar> {
        any::<[u8; 32]>().prop_map(|bytes| Scalar::from_bytes_mod_order(bytes))
    }

    fn keypair() -> impl Strategy<Value = (Scalar, RistrettoPoint)> {
        scalar().prop_map(|s| (s, RISTRETTO_BASEPOINT_POINT * s))
    }

    #[proptest]
    fn test_blind_diffie_hellman_flow(
        #[strategy(keypair())] a: (Scalar, RistrettoPoint),
        secret_message: Vec<u8>,
    ) {
        // Alice initializes
        let (a, a_pub) = a;

        // Bob blinds the secret message
        let (_, r, b_prime) = blind(&secret_message);

        // Alice signs and produces Schnorr signature
        let (c_prime, signature) = sign_blinded(&a, &b_prime);

        // Bob unblinds and verifies Schnorr signature
        let c = unblind_and_verify_signature(&c_prime, &r, &a_pub, &signature).unwrap();

        // Alice verifies the unblinded signature
        prop_assert!(verify_unblinded_point(&a, &secret_message, &c));
    }

    #[proptest]
    fn test_schnorr_signature_tampering(
        #[strategy(keypair())] a: (Scalar, RistrettoPoint),
        secret_message: Vec<u8>,
    ) {
        // Alice initializes
        let (a, a_pub) = a;

        // Bob blinds the secret message
        let (_, r, b_prime) = blind(&secret_message);

        // Alice signs and produces Schnorr signature
        let (c_prime, signature) = sign_blinded(&a, &b_prime);

        // Tamper with the signature
        let tampered_signature = Signature {
            r: signature.r + RISTRETTO_BASEPOINT_POINT,
            s: signature.s,
        };

        // Bob tries to unblind with tampered signature
        let result = unblind_and_verify_signature(&c_prime, &r, &a_pub, &tampered_signature);

        // The unblinding should fail due to invalid Schnorr signature
        prop_assert!(result.is_none());
    }
}
