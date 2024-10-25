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
        // Step 1: Split inputs into 32-bit limbs for modular multiplication
        let mut lhs_limbs = Vec::new();
        let mut rhs_limbs = Vec::new();

        for i in 0..4 {
            // Split each 64-bit element into two 32-bit limbs
            let lhs_bits = builder.split_le_base::<2>(lhs.elements[i], 32);
            let rhs_bits = builder.split_le_base::<2>(rhs.elements[i], 32);

            lhs_limbs.extend(lhs_bits);
            rhs_limbs.extend(rhs_bits);
        }

        // Step 2: Perform schoolbook multiplication of 32-bit limbs
        let mut result_limbs = vec![builder.zero(); 8];
        for i in 0..4 {
            for j in 0..4 {
                let prod = builder.mul(lhs_limbs[i], rhs_limbs[j]);
                let shift = i + j;

                // Add to appropriate position with carry handling
                let mut carry = prod;
                for k in shift..shift + 2 {
                    if k < 8 {
                        let sum = builder.add(result_limbs[k], carry);
                        let divisor = builder.constant(F::from_canonical_u64(1u64 << 32));
                        // Use multiplication and subtraction to compute remainder
                        let div = builder.div(sum, divisor);
                        let div_times_divisor = builder.mul(div, divisor);
                        let rem = builder.sub(sum, div_times_divisor);
                        result_limbs[k] = rem;
                        carry = div;
                    }
                }
            }
        }

        // Step 3: Reduce modulo 2^252 + 27742317777372353535851937790883648493
        let modulus_low = F::from_canonical_u64((1u64 << 32) - 1);
        let modulus_high = F::from_canonical_u64(1u64 << 32);

        // Handle the reduction by using the special form of the modulus
        for i in (4..8).rev() {
            let hi = result_limbs[i];

            // Multiply high limb by 2^32 - 1 and add to lower limbs
            for j in 0..4 {
                let modulus_low_target = builder.constant(modulus_low);
                let term = builder.mul(hi, modulus_low_target);
                result_limbs[j] = builder.add(result_limbs[j], term);
            }

            // Add high limb to next lower limb
            if i > 0 {
                result_limbs[i - 1] = builder.add(result_limbs[i - 1], hi);
            }
        }

        // Combine limbs back into 64-bit elements
        let mut final_elements = [builder.zero(); 4];
        for i in 0..4 {
            let lo = result_limbs[i * 2];
            let modulus_high_target = builder.constant(modulus_high);
            let hi = builder.mul(result_limbs[i * 2 + 1], modulus_high_target);
            final_elements[i] = builder.add(lo, hi);
        }

        HashOutTarget {
            elements: final_elements,
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
    #[ignore]
    fn test_curve25519_scalar_roundtrip(scalar: Scalar) {
        prop_assert_eq!(Scalar::from(DalekScalar::from(scalar)), scalar);
    }

    #[proptest(cases = 1)]
    fn test_mul_scalars(a: Scalar, b: Scalar) {
        prop_assert_eq!(test_circuit_mul(a, b), Ok(()))
    }
}
