use std::{fmt, ops::Mul};

use curve25519_dalek::Scalar as DalekScalar;
use proptest::prelude::*;

use super::circuit_ops::CircuitMul;
use crate::protocol::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Scalar([F; 4]);

impl fmt::Debug for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Scalar")
            .field(&DalekScalar::from(*self).as_bytes())
            .finish()
    }
}

impl Scalar {
    pub fn target(builder: &mut CircuitBuilder) -> HashOutTarget {
        HashOutTarget {
            elements: builder.add_virtual_targets(4).try_into().unwrap(),
        }
    }
}

impl Encode for Scalar {
    fn as_bytes(&self) -> Vec<u8> {
        self.0.iter().flat_map(|&x| x.0.to_le_bytes()).collect()
    }
}

impl EncodeFields for Scalar {
    fn as_fields(&self) -> Vec<F> {
        self.0.to_vec()
    }
}

impl Decode for Scalar {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 32 {
            return Err(Error::DecodeError("Expected 32 bytes".to_string()));
        }

        let mut fields = [F::ZERO; 4];
        for (i, chunk) in bytes.chunks(8).enumerate() {
            fields[i] = F::from_noncanonical_u64(u64::from_le_bytes(chunk.try_into().unwrap()));
        }

        Ok(Self(fields))
    }
}

impl DecodeFields for Scalar {
    fn from_fields(bytes: &[F]) -> Result<Self, Error> {
        if bytes.len() != 4 {
            return Err(Error::DecodeError("Expected 4 field elements".to_string()));
        }

        Ok(Self(bytes.try_into().unwrap()))
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
        let mut result = [zero; 4];

        // Step 1: Compute partial products
        let mut t = [zero; 8];
        for i in 0..4 {
            for j in 0..4 {
                let prod = builder.mul(lhs.elements[i], rhs.elements[j]);
                t[i + j] = builder.add(t[i + j], prod);
            }
        }

        // Step 2: Reduction modulo 2^255 - 19
        let nineteen = F::from_canonical_u64(19);

        // First reduction pass
        for i in (4..8).rev() {
            let reduced = builder.mul_const(nineteen, t[i]);
            t[i - 4] = builder.add(t[i - 4], reduced);
        }

        // Initial copy
        for i in 0..4 {
            result[i] = t[i];
        }

        // Multiple reduction passes to ensure complete reduction
        for _ in 0..3 {
            let mut carry = builder.zero();
            for i in 0..4 {
                let sum = builder.add(result[i], carry);
                let (low, high) = builder.split_low_high(sum, 64, 64);
                result[i] = low;

                if i < 3 {
                    carry = high;
                } else {
                    // Final carry gets multiplied by 19 and added back to first limb
                    let reduced = builder.mul_const(nineteen, high);
                    result[0] = builder.add(result[0], reduced);
                }
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

    crate::test_encode_bytes!(Scalar);
    crate::test_encode_fields!(Scalar);

    fn test_circuit_mul<A, B>(a: A, b: B, verify: bool) -> Result<A, Error>
    where
        A: Arbitrary + EncodeFields + DecodeFields + CircuitMul<B>,
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
        builder.register_public_inputs(&result.elements);

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

        let proof = circuit.prove(pw)?;

        if verify {
            circuit.verify(proof.clone())?;
        }

        let result: &[F] = &proof.public_inputs[(proof.public_inputs.len() - 4)..];

        A::from_fields(&result)
    }

    #[proptest]
    fn test_curve25519_conversion_roundtrip(scalar: Scalar) {
        prop_assert_eq!(Scalar::from(DalekScalar::from(scalar)), scalar);
    }

    #[proptest(cases = 1)]
    fn test_mul_scalars(a: Scalar, b: Scalar) {
        let val = DalekScalar::from(a) * DalekScalar::from(b);

        prop_assert_eq!(
            unwind_panic(move || test_circuit_mul(a, b, false)),
            Ok(Scalar::from(val))
        );
    }
}
