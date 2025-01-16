use std::fmt;

use curve25519_dalek::{ristretto::CompressedRistretto, RistrettoPoint, Scalar};
use rand::{CryptoRng, Rng};
use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::Error;

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
        Self(rng.gen())
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

#[cfg(test)]
mod tests {
    use curve25519_dalek::{RistrettoPoint, Scalar};
    use proptest::prelude::*;
    use rand::prelude::*;
    use rand_chacha::ChaCha20Rng;
    use test_strategy::proptest;

    use super::Bytes;

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
