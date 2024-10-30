use std::fmt;

use curve25519_dalek::{ristretto::CompressedRistretto, RistrettoPoint, Scalar};
use plonky2::{hash::hash_types::HashOut, plonk::config::GenericHashOut};
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::{protocol::circuit::*, Decode, DecodeFields, Encode, EncodeFields, Error};

pub type Hash = Bytes<32>;
pub type BlindSignature = Bytes<32>;
pub type BlindedValue = Bytes<32>;
pub type PublicKey = Bytes<32>;
pub type SecretKey = Bytes<32>;
pub type Signature = Bytes<32>;
pub type Name = Bytes<32>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Arbitrary, Hash)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Bytes<const N: usize>(
    #[filter(#0 != [0u8; N])]
    #[serde(with = "serde_bytes")]
    [u8; N],
);

impl<const N: usize> Bytes<N> {
    #[inline]
    pub const fn zero() -> Self {
        Self([0u8; N])
    }

    #[inline]
    pub const fn new(val: [u8; N]) -> Self {
        Self(val)
    }

    #[inline]
    pub const fn is_zero(&self) -> bool {
        let mut i = 0;

        while i < N {
            if self.0[i] != 0 {
                return false;
            }

            i += 1;
        }

        true
    }

    #[inline]
    pub const fn inner(&self) -> [u8; N] {
        self.0
    }

    #[inline]
    pub const fn inner_mut(&mut self) -> &mut [u8; N] {
        &mut self.0
    }

    #[inline]
    pub fn random<R: CryptoRng + Rng>(rng: &mut R) -> Bytes<N> {
        if N == 32 {
            Bytes::from_bytes(&Scalar::random(rng).to_bytes()).unwrap()
        } else {
            Self(rng.gen())
        }
    }
}

impl<const N: usize> Default for Bytes<N> {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl<const N: usize> From<[u8; N]> for Bytes<N> {
    #[inline]
    fn from(value: [u8; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize> From<[u64; 4]> for Bytes<N> {
    #[inline]
    fn from(value: [u64; 4]) -> Self {
        let mut bytes = [0u8; N];

        for (i, &val) in value.iter().enumerate() {
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&val.to_le_bytes());
        }

        Self(bytes)
    }
}

impl<const N: usize> From<&[u64]> for Bytes<N> {
    #[inline]
    fn from(slice: &[u64]) -> Self {
        let mut bytes = [0u8; N];

        for (i, &val) in slice.iter().enumerate().take(4) {
            let start = i * 8;
            let end = (i + 1) * 8;

            if end <= bytes.len() {
                bytes[start..end].copy_from_slice(&val.to_le_bytes());
            } else {
                break;
            }
        }

        Self(bytes)
    }
}

impl<const N: usize> AsRef<[u8]> for Bytes<N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<Bytes<32>> for Scalar {
    type Error = Error;

    #[inline]
    fn try_from(bytes: Bytes<32>) -> Result<Self, Error> {
        Scalar::from_canonical_bytes(bytes.inner())
            .into_option()
            .ok_or(Error::DecodeError(format!("Invalid scalar: {}", bytes)))
    }
}

impl From<Scalar> for Bytes<32> {
    #[inline]
    fn from(scalar: Scalar) -> Self {
        Self(scalar.to_bytes())
    }
}

impl From<RistrettoPoint> for Bytes<32> {
    #[inline]
    fn from(point: RistrettoPoint) -> Self {
        Self(point.compress().to_bytes())
    }
}

impl TryFrom<Bytes<32>> for RistrettoPoint {
    type Error = Error;

    #[inline]
    fn try_from(bytes: Bytes<32>) -> Result<Self, Self::Error> {
        CompressedRistretto::from_slice(&bytes.0)
            .map_err(|e| Error::DecodeError(e.to_string()))?
            .decompress()
            .ok_or(Error::DecodeError("Could not decompress point".to_string()))
    }
}

impl<const N: usize> fmt::Display for Bytes<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl<const N: usize> fmt::Debug for Bytes<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<HashOut<F>> for Bytes<32> {
    #[inline]
    fn from(value: HashOut<F>) -> Self {
        Self::from_bytes(&value.to_bytes()).unwrap()
    }
}

impl From<Bytes<32>> for HashOut<F> {
    #[inline]
    fn from(value: Bytes<32>) -> Self {
        HashOut {
            elements: value.as_fields().try_into().unwrap(),
        }
    }
}

impl<const N: usize> Encode for Bytes<N> {
    #[inline]
    fn as_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl<const N: usize> EncodeFields for Bytes<N> {
    #[inline]
    fn as_fields(&self) -> Vec<F> {
        self.0
            .chunks(8)
            .map(|chunk| F::from_canonical_u64(u64::from_le_bytes(chunk.try_into().unwrap())))
            .collect()
    }
}

impl<const N: usize> Decode for Bytes<N> {
    #[inline]
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != N {
            return Err(Error::DecodeError(format!(
                "Invalid length for Bytes: expected N, got {}",
                bytes.len()
            )));
        }

        let mut array = [0u8; N];
        array.copy_from_slice(bytes);
        Ok(Self(array))
    }
}

impl<const N: usize> DecodeFields for Bytes<N> {
    #[inline]
    fn from_fields(fields: &[F]) -> Result<Self, Error> {
        if fields.len() != N / 8 {
            return Err(Error::DecodeError(format!(
                "Invalid number of fields for Bytes: expected {}, got {}",
                N / 8,
                fields.len()
            )));
        }

        let mut bytes = [0u8; N];
        for (i, field) in fields.iter().enumerate() {
            let field_bytes = field.to_canonical_u64().to_le_bytes();
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&field_bytes);
        }

        Ok(Self(bytes))
    }
}

#[cfg(test)]
mod tests {
    use curve25519_dalek::{RistrettoPoint, Scalar};
    use proptest::prelude::*;
    use rand::prelude::*;
    use rand_chacha::ChaCha20Rng;
    use test_strategy::proptest;

    use super::Bytes;
    use crate::{test_encode_bytes, test_encode_fields};

    pub type Bytes8 = Bytes<8>;
    test_encode_bytes!(Bytes8);
    test_encode_fields!(Bytes8);

    pub type Bytes16 = Bytes<16>;
    test_encode_bytes!(Bytes16);
    test_encode_fields!(Bytes16);

    pub type Bytes32 = Bytes<32>;
    test_encode_bytes!(Bytes32);
    test_encode_fields!(Bytes32);

    pub type Bytes64 = Bytes<64>;
    test_encode_bytes!(Bytes64);
    test_encode_fields!(Bytes64);

    fn crypto_rng() -> impl Strategy<Value = ChaCha20Rng> {
        any::<[u8; 32]>().prop_map(ChaCha20Rng::from_seed)
    }

    fn scalar() -> impl Strategy<Value = Scalar> {
        crypto_rng().prop_map(|mut rng| Scalar::random(&mut rng))
    }

    fn point() -> impl Strategy<Value = RistrettoPoint> {
        crypto_rng().prop_map(|mut rng| RistrettoPoint::random(&mut rng))
    }

    #[proptest]
    fn test_bytes32_rng_scalar_conversion(#[strategy(scalar())] scalar: Scalar) {
        prop_assert_eq!(Scalar::try_from(Bytes::<32>::from(scalar))?, scalar);
    }

    #[proptest]
    fn test_bytes32_rng_point_conversion(#[strategy(point())] point: RistrettoPoint) {
        prop_assert_eq!(RistrettoPoint::try_from(Bytes::<32>::from(point))?, point);
    }

    #[proptest]
    fn test_bytes32_scalar_conversion(mut bytes: Bytes<32>) {
        bytes.inner_mut()[31] = 0;

        prop_assert_eq!(Bytes::from(Scalar::try_from(bytes)?), bytes);
    }

    #[proptest]
    #[should_panic]
    fn test_bytes32_scalar_conversion_point_outside_of_mod(mut bytes: Bytes<32>) {
        bytes.inner_mut()[31] = u8::MAX;
        Scalar::try_from(bytes)?;
    }
}
