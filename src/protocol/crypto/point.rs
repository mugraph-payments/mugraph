use curve25519_dalek::{
    ristretto::CompressedRistretto,
    RistrettoPoint as DalekPoint,
    Scalar as DalekScalar,
};
use proptest::prelude::*;

use super::Scalar;
use crate::protocol::{circuit::*, crypto::G};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point([F; 8]);

impl Arbitrary for Point {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        any::<Scalar>()
            .prop_map(|x| Self::from(DalekScalar::from(x) * G))
            .boxed()
    }
}

impl From<DalekPoint> for Point {
    fn from(value: DalekPoint) -> Self {
        let bytes = value.compress().to_bytes();

        Self([
            F::from_noncanonical_u64(u64::from_le_bytes(bytes[0..8].try_into().unwrap())),
            F::from_noncanonical_u64(u64::from_le_bytes(bytes[8..16].try_into().unwrap())),
            F::from_noncanonical_u64(u64::from_le_bytes(bytes[16..24].try_into().unwrap())),
            F::from_noncanonical_u64(u64::from_le_bytes(bytes[24..32].try_into().unwrap())),
            F::ZERO,
            F::ZERO,
            F::ZERO,
            F::ZERO,
        ])
    }
}

impl From<Point> for DalekPoint {
    fn from(value: Point) -> Self {
        let mut bytes = [0u8; 32];

        for (i, field) in value.0.iter().take(4).enumerate() {
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&field.0.to_le_bytes());
        }

        CompressedRistretto(bytes).decompress().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use curve25519_dalek::RistrettoPoint as DalekPoint;
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn test_curve25519_point_roundtrip(point: Point) {
        prop_assert_eq!(Point::from(DalekPoint::from(point)), point);
    }
}
