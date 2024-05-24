use std::fmt::{Debug, Display};

use ed25519_dalek::{
    Signature as Ed25519Signature, Verifier as _, VerifyingKey as Ed25519PublicKey,
};
use proptest::prelude::*;

use crate::prelude::*;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pubkey([u8; 32]);

impl_associate_bytes_types!(Pubkey);

impl Pubkey {
    pub fn verify(&self, data: &[u8], signature: &Signature) -> Result<()> {
        let signature = Ed25519Signature::from_slice(&signature.to_bytes())?;
        let key = Ed25519PublicKey::from_bytes(&self.to_bytes())?;

        key.verify(data, &signature)?;

        Ok(())
    }
}

impl Debug for Pubkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Pubkey").field(&self.to_hex()).finish()
    }
}

impl Display for Pubkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl ToBytes for Pubkey {
    type Output = [u8; 32];

    fn to_bytes(&self) -> Self::Output {
        self.0
    }
}

impl FromBytes for Pubkey {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let bytes: [u8; 32] = bytes.try_into().map_err(|e| {
            Error::FailedDeserialization(format!(
                "failed to get public key for `{}`, expected 32 bytes but got {}",
                hex::encode(bytes),
                e
            ))
        })?;

        Ok(Ed25519PublicKey::from_bytes(&bytes)?.into())
    }
}

impl From<Ed25519PublicKey> for Pubkey {
    fn from(pk: Ed25519PublicKey) -> Self {
        Self(*pk.as_bytes())
    }
}

impl Arbitrary for Pubkey {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        any::<Account>().prop_map(|x| x.pubkey()).boxed()
    }
}

#[cfg(test)]
mod tests {
    use test_strategy::proptest;

    use crate::prelude::*;

    test_to_bytes!(Pubkey);

    #[proptest(fork = false)]
    fn test_dalek_roundtrip(a: Pubkey) {
        let as_dalek = ed25519_dalek::VerifyingKey::from_bytes(&a.to_bytes())?;
        let b: Pubkey = as_dalek.into();

        assert_eq!(a, b);
    }

    #[proptest(fork = false)]
    fn test_display_identity(a: Pubkey, b: Pubkey) {
        assert_eq!(a.to_string() == b.to_string(), a == b);
    }

    #[proptest(fork = false)]
    fn test_debug_identity(a: Pubkey, b: Pubkey) {
        assert_eq!(format!("{:?}", a) == format!("{:?}", b), a == b);
    }
}
