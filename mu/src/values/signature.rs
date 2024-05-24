use std::fmt::{Debug, Display};

use proptest::prelude::*;

use crate::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Signature([u8; 64]);

impl_associate_bytes_types!(Signature);

impl Signature {
    pub fn new(account: &Account, data: &[u8]) -> Self {
        account.sign(data)
    }
}

impl Default for Signature {
    fn default() -> Self {
        Self([0u8; 64])
    }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Signature").field(&self.to_hex()).finish()
    }
}

impl Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl FromBytes for Signature {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let bytes: [u8; 64] = bytes.try_into().map_err(|e| {
            Error::FailedDeserialization(format!(
                "failed to get signature for `{}`, expected 64 bytes but got {}",
                hex::encode(bytes),
                e
            ))
        })?;

        Ok(ed25519_dalek::Signature::from_slice(&bytes)?.into())
    }
}

impl ToBytes for Signature {
    type Output = [u8; 64];

    fn to_bytes(&self) -> Self::Output {
        self.0
    }
}

impl From<ed25519_dalek::Signature> for Signature {
    fn from(pk: ed25519_dalek::Signature) -> Self {
        Self(pk.to_bytes())
    }
}

impl Arbitrary for Signature {
    type Parameters = Vec<u8>;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(data: Self::Parameters) -> Self::Strategy {
        any::<Account>()
            .prop_map(move |acc| Self::new(&acc, &data))
            .boxed()
    }

    fn arbitrary() -> Self::Strategy {
        any::<Vec<u8>>()
            .prop_flat_map(Signature::arbitrary_with)
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use proptest::prelude::*;
    use test_strategy::proptest;

    test_to_bytes!(Signature);

    #[proptest(fork = false)]
    fn test_new_always_generates_valid_signatures(data: Vec<u8>) {
        let account = Account::new();
        let signature = Signature::new(&account, &data);

        prop_assert_eq!(account.pubkey().verify(&data, &signature), Ok(()));
    }

    #[proptest(fork = false)]
    fn test_dalek_roundtrip(a: Signature) {
        let as_dalek = ed25519_dalek::Signature::from_bytes(&a.to_bytes());
        let b: Signature = as_dalek.into();

        assert_eq!(a, b);
    }

    #[proptest(fork = false)]
    fn test_display_identity(a: Signature, b: Signature) {
        assert_eq!(a.to_string() == b.to_string(), a == b);
    }

    #[proptest(fork = false)]
    fn test_debug_identity(a: Signature, b: Signature) {
        assert_eq!(format!("{:?}", a) == format!("{:?}", b), a == b);
    }
}
