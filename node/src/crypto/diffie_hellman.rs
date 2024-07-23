use curve25519_dalek::{constants::RISTRETTO_BASEPOINT_POINT, traits::MultiscalarMul};
use rand::rngs::OsRng;

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
) -> (RistrettoPoint, Scalar, Scalar) {
    let signed_point = blinded_point * private_key;

    // Generate DLEQ proof
    let random_scalar = Scalar::random(&mut OsRng);
    let r1 = RISTRETTO_BASEPOINT_POINT * random_scalar;
    let r2 = blinded_point * random_scalar;

    let challenge = hash_to_scalar(&[
        DLEQ_DOMAIN_SEPARATOR,
        &r1.compress().to_bytes(),
        &r2.compress().to_bytes(),
        &(RISTRETTO_BASEPOINT_POINT * private_key)
            .compress()
            .to_bytes(),
        &signed_point.compress().to_bytes(),
    ]);
    let response = random_scalar + challenge * private_key;

    (signed_point, challenge, response)
}

pub fn unblind_verify(
    signed_point: &RistrettoPoint,
    blinding_factor: &Scalar,
    public_key: &RistrettoPoint,
    blinded_point: &RistrettoPoint,
    challenge: &Scalar,
    response: &Scalar,
) -> Option<RistrettoPoint> {
    // Verify DLEQ proof
    let r1 = RistrettoPoint::multiscalar_mul(
        &[*response, -*challenge],
        &[RISTRETTO_BASEPOINT_POINT, *public_key],
    );
    let r2 = RistrettoPoint::multiscalar_mul(
        &[*response, -*challenge],
        &[*blinded_point, *signed_point],
    );

    let computed_challenge = hash_to_scalar(&[
        DLEQ_DOMAIN_SEPARATOR,
        &r1.compress().to_bytes(),
        &r2.compress().to_bytes(),
        &public_key.compress().to_bytes(),
        &signed_point.compress().to_bytes(),
    ]);

    if computed_challenge == *challenge {
        Some(signed_point - (public_key * blinding_factor))
    } else {
        None
    }
}

pub fn verify_unblinded(
    private_key: &Scalar,
    message: &[u8],
    unblinded_point: &RistrettoPoint,
) -> bool {
    let y = hash_to_curve(message);
    &y * private_key == *unblinded_point
}

pub fn pedersen_commit(value: Scalar, blinding_factor: Scalar) -> RistrettoPoint {
    let h = hash_to_curve(b"PEDERSEN_H");
    RISTRETTO_BASEPOINT_POINT * value + h * blinding_factor
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

        // Alice signs and produces DLEQ proof
        let (c_prime, e, s) = sign_blinded(&a, &b_prime);

        // Bob unblinds and verifies DLEQ proof
        let c = unblind_verify(&c_prime, &r, &a_pub, &b_prime, &e, &s).unwrap();

        // Alice verifies the unblinded signature
        prop_assert!(verify_unblinded(&a, &secret_message, &c));
    }

    #[proptest]
    fn test_dleq_proof_tampering(
        #[strategy(keypair())] a: (Scalar, RistrettoPoint),
        secret_message: Vec<u8>,
    ) {
        // Alice initializes
        let (a, a_pub) = a;

        // Bob blinds the secret message
        let (_, r, b_prime) = blind(&secret_message);

        // Alice signs and produces DLEQ proof
        let (c_prime, e, s) = sign_blinded(&a, &b_prime);

        // Tamper with c_prime
        let tampered_c_prime = c_prime + RISTRETTO_BASEPOINT_POINT;

        // Bob tries to unblind with tampered c_prime
        let result = unblind_verify(&tampered_c_prime, &r, &a_pub, &b_prime, &e, &s);

        // The unblinding should fail due to invalid DLEQ proof
        prop_assert!(result.is_none());
    }
}
