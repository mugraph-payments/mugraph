use std::ops::Mul;

use curve25519_dalek::Scalar as DalekScalar;
use proptest::prelude::*;

use super::circuit_ops::CircuitMul;
use crate::protocol::*;

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
        let mut result = [builder.zero(); 8];

        // Perform multi-limb multiplication
        for i in 0..4 {
            for j in 0..4 {
                let product = builder.mul(lhs.elements[i], rhs.elements[j]);
                result[i + j] = builder.add(result[i + j], product);
            }
        }

        // Handle carry propagation
        let modulus = builder.constant(F::from_noncanonical_biguint(F::order()));

        for i in 0..7 {
            let quotient = builder.div(result[i], modulus);
            let mul = builder.mul(quotient, modulus);
            let remainder = builder.sub(result[i], mul);

            result[i] = remainder;
            result[i + 1] = builder.add(result[i + 1], quotient);
        }

        // Final reduction
        let quotient = builder.div(result[7], modulus);
        let mul = builder.mul(quotient, modulus);
        let final_result = builder.sub(result[7], mul);

        // Combine the final 4 limbs into a HashOutTarget
        HashOutTarget {
            elements: [result[0], result[1], result[2], final_result],
        }
    }
}

#[cfg(test)]
mod tests {
    use curve25519_dalek::Scalar as DalekScalar;
    use plonky2::hash::hash_types::HashOut;
    use test_strategy::proptest;

    use super::*;
    use crate::unwind_panic;

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

    #[proptest]
    fn test_curve25519_scalar_roundtrip(scalar: Scalar) {
        prop_assert_eq!(Scalar::from(DalekScalar::from(scalar)), scalar);
    }

    #[proptest]
    fn test_mul_scalars(a: Scalar, b: Scalar) {
        prop_assert_eq!(test_circuit_mul(a, b), Ok(()))
    }
}
