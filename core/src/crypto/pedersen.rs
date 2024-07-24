use super::*;

pub fn pedersen_commit(
    value: Scalar,
    blinding_factor: Scalar,
    h: RistrettoPoint,
) -> RistrettoPoint {
    RISTRETTO_BASEPOINT_POINT * value + h * blinding_factor
}

pub fn pedersen_verify(
    commitment: RistrettoPoint,
    value: Scalar,
    blinding_factor: Scalar,
    h: RistrettoPoint,
) -> bool {
    let computed_commitment = pedersen_commit(value, blinding_factor, h);
    commitment == computed_commitment
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rand::rngs::OsRng;

    fn scalar() -> impl Strategy<Value = Scalar> {
        Just(Scalar::random(&mut OsRng))
    }

    fn point() -> impl Strategy<Value = RistrettoPoint> {
        Just(RistrettoPoint::random(&mut OsRng))
    }

    #[test_strategy::proptest]
    fn test_pedersen_commit_and_verify(
        #[strategy(scalar())] value: Scalar,
        #[strategy(scalar())] blinding_factor: Scalar,
        #[strategy(point())] h: RistrettoPoint,
    ) {
        let commitment = pedersen_commit(value, blinding_factor, h);
        prop_assert!(pedersen_verify(commitment, value, blinding_factor, h));
    }

    #[test_strategy::proptest]
    fn test_pedersen_verify_fails_with_wrong_value(
        #[strategy(scalar())] value: Scalar,
        #[strategy(scalar())] blinding_factor: Scalar,
        #[strategy(point())] h: RistrettoPoint,
        #[strategy(scalar())] wrong_value: Scalar,
    ) {
        let commitment = pedersen_commit(value, blinding_factor, h);
        prop_assert!(!pedersen_verify(
            commitment,
            wrong_value,
            blinding_factor,
            h
        ));
    }

    #[test_strategy::proptest]
    fn test_pedersen_verify_fails_with_wrong_blinding_factor(
        #[strategy(scalar())] value: Scalar,
        #[strategy(scalar())] blinding_factor: Scalar,
        #[strategy(point())] h: RistrettoPoint,
        #[strategy(scalar())] wrong_blinding_factor: Scalar,
    ) {
        let commitment = pedersen_commit(value, blinding_factor, h);
        prop_assert!(!pedersen_verify(
            commitment,
            value,
            wrong_blinding_factor,
            h
        ));
    }

    #[test_strategy::proptest]
    fn test_pedersen_homomorphic_property(
        #[strategy(scalar())] value1: Scalar,
        #[strategy(scalar())] value2: Scalar,
        #[strategy(scalar())] blinding_factor1: Scalar,
        #[strategy(scalar())] blinding_factor2: Scalar,
        #[strategy(point())] h: RistrettoPoint,
    ) {
        let commitment1 = pedersen_commit(value1, blinding_factor1, h);
        let commitment2 = pedersen_commit(value2, blinding_factor2, h);

        let sum_commitment = commitment1 + commitment2;
        let sum_value = value1 + value2;
        let sum_blinding_factor = blinding_factor1 + blinding_factor2;

        let expected_sum_commitment = pedersen_commit(sum_value, sum_blinding_factor, h);

        prop_assert_eq!(sum_commitment, expected_sum_commitment);
    }
}
