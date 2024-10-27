use std::{fmt, ops::Mul};

use curve25519_dalek::Scalar as DalekScalar;
use proptest::prelude::*;

use crate::{protocol::circuit::*, Decode, DecodeFields, Encode, EncodeFields, Error};

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

#[cfg(test)]
mod tests {
    use curve25519_dalek::Scalar as DalekScalar;
    use test_strategy::proptest;

    use super::*;

    crate::test_encode_bytes!(Scalar);
    crate::test_encode_fields!(Scalar);

    #[proptest]
    fn test_curve25519_conversion_roundtrip(scalar: Scalar) {
        prop_assert_eq!(Scalar::from(DalekScalar::from(scalar)), scalar);
    }
}
