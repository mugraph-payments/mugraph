use std::fmt;

use curve25519_dalek::{constants::RISTRETTO_BASEPOINT_POINT as G, RistrettoPoint, Scalar};
use proptest::prelude::*;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::{
    protocol::{circuit::*, *},
    DecodeFields,
    EncodeFields,
    Error,
};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct SecretKey(#[serde(with = "hex::serde")] [u8; 32]);

impl SecretKey {
    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    pub fn random() -> Self {
        Scalar::random(&mut OsRng).into()
    }

    pub fn public(&self) -> PublicKey {
        let this = Scalar::from_bytes_mod_order(self.0);

        (this * G).into()
    }

    /// Signs a blinded note value using this secret key
    ///
    /// This function performs blind signing, which is a cryptographic operation
    /// where the signer (the mint) signs a message (the blinded note value)
    /// without knowing its contents.
    ///
    /// # Arguments
    ///
    /// * `secret_key` - The mint's secret signing key.
    /// * `b_prime` - The blinded note value (B') to be signed.
    ///
    /// # Returns
    ///
    /// Returns C', the signed blinded note as an `RistrettoPoint`.
    pub fn sign_blinded(&self, b_prime: BlindedValue) -> BlindSignature {
        (Scalar::from(*self) * RistrettoPoint::from(b_prime)).into()
    }
}

impl Encode for SecretKey {
    fn as_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl EncodeFields for SecretKey {
    fn as_fields(&self) -> Vec<F> {
        self.0
            .chunks(8)
            .map(|chunk| F::from_canonical_u64(u64::from_le_bytes(chunk.try_into().unwrap())))
            .collect()
    }
}

impl Decode for SecretKey {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 32 {
            return Err(Error::DecodeError(format!(
                "Invalid length for SecretKey: expected 32, got {}",
                bytes.len()
            )));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(bytes);
        Ok(Self(array))
    }
}

impl DecodeFields for SecretKey {
    fn from_fields(fields: &[F]) -> Result<Self, Error> {
        if fields.len() != 4 {
            return Err(Error::DecodeError(format!(
                "Invalid number of fields for SecretKey: expected 4, got {}",
                fields.len()
            )));
        }

        let mut bytes = [0u8; 32];
        for (i, field) in fields.iter().enumerate() {
            let field_bytes = field.to_canonical_u64().to_le_bytes();
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&field_bytes);
        }

        Ok(Self(bytes))
    }
}

impl From<Hash> for SecretKey {
    fn from(value: Hash) -> Self {
        Self(value.inner())
    }
}

impl From<SecretKey> for Hash {
    fn from(value: SecretKey) -> Self {
        Hash::new(value.0)
    }
}

impl From<[u8; 32]> for SecretKey {
    fn from(value: [u8; 32]) -> Self {
        Self(value)
    }
}

impl From<&[u8]> for SecretKey {
    fn from(slice: &[u8]) -> Self {
        if slice.len() != 32 {
            panic!(
                "Invalid slice length for SecretKey: expected 32, got {}",
                slice.len()
            );
        }

        let mut hash = [0u8; 32];
        hash.copy_from_slice(slice);

        Self(hash)
    }
}

impl From<Scalar> for SecretKey {
    fn from(scalar: Scalar) -> Self {
        Self(scalar.to_bytes())
    }
}

impl From<SecretKey> for Scalar {
    fn from(secret_key: SecretKey) -> Self {
        Scalar::from_bytes_mod_order(secret_key.0)
    }
}

impl AsRef<[u8]> for SecretKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Arbitrary for SecretKey {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(params: Self::Parameters) -> Self::Strategy {
        Hash::arbitrary_with(params)
            .prop_map(|x| Self(x.as_ref().try_into().unwrap()))
            .boxed()
    }
}

impl fmt::Display for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::SecretKey;

    crate::test_encode_bytes!(SecretKey);
    crate::test_encode_fields!(SecretKey);
}
