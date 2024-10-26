use std::{fmt, ops::Mul};

use curve25519_dalek::Scalar as DalekScalar;
use proptest::prelude::*;

use super::circuit_ops::CircuitMul;
use crate::protocol::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Scalar([F; 4]);

impl fmt::Debug for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scalar(0x")?;
        for field in self.0.iter().rev() {
            write!(f, "{:016x}", field.0)?;
        }
        write!(f, ")")
    }
}

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
            F::from_noncanonical_u64(u64::from_le_bytes(bytes[0..8].try_into().unwrap())),
            F::from_noncanonical_u64(u64::from_le_bytes(bytes[8..16].try_into().unwrap())),
            F::from_noncanonical_u64(u64::from_le_bytes(bytes[16..24].try_into().unwrap())),
            F::from_noncanonical_u64(u64::from_le_bytes(bytes[24..32].try_into().unwrap())),
        ])
    }
}

impl From<Scalar> for DalekScalar {
    fn from(value: Scalar) -> Self {
        let mut bytes = [0u8; 32];

        for (i, field) in value.0.iter().enumerate() {
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&field.0.to_le_bytes());
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
        let zero = builder.zero();

        // Create intermediate targets for partial products
        let mut partial_sums = vec![zero; 8];

        // Process each limb pair and handle carries
        for i in 0..4 {
            for j in 0..4 {
                let idx = i + j;
                partial_sums[idx] =
                    builder.mul_add(lhs.elements[i], rhs.elements[j], partial_sums[idx]);
            }
        }

        // Perform carry propagation
        let mut result = [zero; 4];
        let mut carry = zero;

        for i in 0..4 {
            let sum_with_carry = builder.add(partial_sums[i], carry);

            // Constrain sum_with_carry to be within 65 bits and get the new carry
            let (sum_low, new_carry) = builder.split_low_high(sum_with_carry, 64, 65);
            result[i] = sum_low;

            if i < 3 {
                carry = builder.add(partial_sums[i + 4], new_carry);
            }
        }

        HashOutTarget { elements: result }
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
    #[ignore]
    fn test_curve25519_scalar_roundtrip(scalar: Scalar) {
        prop_assert_eq!(Scalar::from(DalekScalar::from(scalar)), scalar);
    }

    #[proptest(cases = 1)]
    fn test_mul_scalars(a: Scalar, b: Scalar) {
        prop_assert_eq!(test_circuit_mul(a, b), Ok(()))
    }
}
