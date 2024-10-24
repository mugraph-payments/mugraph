mod circuit_ops;

use std::ops::Mul;

use circuit_ops::CircuitMul;
use curve25519_dalek::{
    ristretto::CompressedRistretto,
    RistrettoPoint as DalekPoint,
    Scalar as DalekScalar,
};
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::{HashOut, RichField},
};
use proptest::prelude::*;

use crate::{protocol::*, unwind_panic};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scalar([F; 4]);

impl Scalar {
    pub fn target(builder: &mut CircuitBuilder) -> HashOutTarget {
        HashOutTarget {
            elements: builder.add_virtual_targets(4).try_into().unwrap(),
        }
    }
}

impl EncodeFields for Scalar {
    fn as_fields(&self) -> Vec<F> {
        self.0.to_vec()
    }
}

impl Arbitrary for Scalar {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        any::<[u8; 32]>()
            .prop_map(DalekScalar::from_bytes_mod_order)
            .prop_map(Self::from)
            .boxed()
    }
}

impl From<DalekScalar> for Scalar {
    fn from(value: DalekScalar) -> Self {
        let bytes = value.to_bytes();

        Self([
            F::from_canonical_u64(u64::from_le_bytes(bytes[0..8].try_into().unwrap())),
            F::from_canonical_u64(u64::from_le_bytes(bytes[8..16].try_into().unwrap())),
            F::from_canonical_u64(u64::from_le_bytes(bytes[16..24].try_into().unwrap())),
            F::from_canonical_u64(u64::from_le_bytes(bytes[24..32].try_into().unwrap())),
        ])
    }
}

impl From<Scalar> for DalekScalar {
    fn from(value: Scalar) -> Self {
        let mut bytes = [0u8; 32];

        for (i, field) in value.0.iter().enumerate() {
            let value = field.to_canonical_u64();
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&value.to_le_bytes());
        }

        DalekScalar::from_bytes_mod_order(bytes)
    }
}

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
            F::from_canonical_u64(u64::from_le_bytes(bytes[0..8].try_into().unwrap())),
            F::from_canonical_u64(u64::from_le_bytes(bytes[8..16].try_into().unwrap())),
            F::from_canonical_u64(u64::from_le_bytes(bytes[16..24].try_into().unwrap())),
            F::from_canonical_u64(u64::from_le_bytes(bytes[24..32].try_into().unwrap())),
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
            let value = field.to_canonical_u64();
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&value.to_le_bytes());
        }

        CompressedRistretto(bytes).decompress().unwrap()
    }
}

impl Mul<Scalar> for Scalar {
    type Output = Scalar;

    fn mul(self, rhs: Scalar) -> Self::Output {
        (DalekScalar::from(self) * DalekScalar::from(rhs)).into()
    }
}

impl CircuitMul<Scalar> for Scalar {
    fn circuit_mul(
        builder: &mut CircuitBuilder,
        lhs: HashOutTarget,
        rhs: HashOutTarget,
    ) -> HashOutTarget {
        todo!()
    }
}

fn test_circuit_mul<A, B>(a: A, b: B) -> Result<(), Error>
where
    A: Arbitrary + EncodeFields + CircuitMul<B>,
    B: Arbitrary + EncodeFields,
{
    let mut builder = circuit_builder();

    let a_target = builder.add_virtual_hash();
    builder.register_public_inputs(&a_target.elements);
    let b_target = builder.add_virtual_hash();
    builder.register_public_inputs(&b_target.elements);
    let result = builder.add_virtual_hash();

    let c = Scalar::circuit_mul(&mut builder, a_target, b_target);
    builder.register_public_inputs(&c.elements);

    builder.connect_hashes(result, c);
    let circuit = builder.build::<C>();

    let mut pw = PartialWitness::new();
    pw.set_hash_target(
        a_target,
        HashOut {
            elements: a.as_fields().try_into().unwrap(),
        },
    );
    pw.set_hash_target(
        b_target,
        HashOut {
            elements: b.as_fields().try_into().unwrap(),
        },
    );
    pw.set_hash_target(
        result,
        HashOut {
            elements: (a * b).as_fields().try_into().unwrap(),
        },
    );

    let proof = unwind_panic!(circuit.prove(pw).map_err(|e| Error::CryptoError {
        kind: e.root_cause().to_string(),
        reason: e.to_string(),
    }))?;

    unwind_panic!(circuit.verify(proof).map_err(|e| Error::CryptoError {
        kind: e.root_cause().to_string(),
        reason: e.to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use curve25519_dalek::{RistrettoPoint as DalekPoint, Scalar as DalekScalar};
    use test_strategy::proptest;

    use super::*;

    #[proptest]
    fn test_curve25519_scalar_roundtrip(scalar: Scalar) {
        prop_assert_eq!(Scalar::from(DalekScalar::from(scalar)), scalar);
    }

    #[proptest]
    fn test_curve25519_point_roundtrip(scalar: Point) {
        prop_assert_eq!(Point::from(DalekPoint::from(scalar)), scalar);
    }

    #[proptest]
    fn test_mul_scalars(a: Scalar, b: Scalar) {
        prop_assert_eq!(test_circuit_mul(a, b), Ok(()))
    }
}
