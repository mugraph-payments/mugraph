use std::fmt::{Debug, Formatter};

use ed25519_dalek::{Signer, SigningKey, SECRET_KEY_LENGTH};
use proptest::prelude::*;

use crate::prelude::*;

pub struct Account {
    signing_key: SigningKey,
}

impl Account {
    pub fn new() -> Self {
        Self {
            signing_key: SigningKey::generate(&mut rand::thread_rng()),
        }
    }

    pub fn pubkey(&self) -> Pubkey {
        self.signing_key.verifying_key().into()
    }

    pub fn sign(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data).into()
    }

    pub fn verify(&self, data: &[u8], signature: &Signature) -> Result<()> {
        self.pubkey().verify(data, signature)
    }
}

impl Debug for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Account")
            .field("pubkey", &self.signing_key.verifying_key())
            .finish()
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.pubkey() == other.pubkey()
    }
}

impl FromBytes for Account {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            signing_key: SigningKey::from_bytes(bytes.try_into()?),
        })
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

impl Arbitrary for Account {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        any::<[u8; SECRET_KEY_LENGTH]>()
            .prop_map(|data| Self::from_bytes(&data).unwrap())
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_strategy::proptest;

    #[proptest(fork = false)]
    fn test_account_sign_and_verify(account: Account, data: Vec<u8>) {
        let signature = account.sign(&data);
        assert_eq!(account.verify(&data, &signature), Ok(()));
    }

    #[proptest(fork = false)]
    fn test_equality_identity(acc: Account) {
        prop_assert_eq!(&acc, &acc);
    }

    #[proptest(fork = false)]
    fn test_inequality_when_secret_key_is_different(
        a: [u8; SECRET_KEY_LENGTH],
        b: [u8; SECRET_KEY_LENGTH],
    ) {
        prop_assume!(a != b);

        prop_assert_ne!(Account::from_bytes(&a), Account::from_bytes(&b));
    }

    #[proptest(fork = false)]
    fn test_different_data_have_different_signatures(
        account: Account,
        data_a: Vec<u8>,
        data_b: Vec<u8>,
    ) {
        prop_assume!(data_a != data_b);

        let signature_a = account.sign(&data_a);
        let signature_b = account.sign(&data_b);

        assert!(signature_a != signature_b);
    }

    #[proptest(fork = false)]
    fn test_only_the_owner_key_can_verify_a_signature(
        account_a: Account,
        account_b: Account,
        data_a: Vec<u8>,
        data_b: Vec<u8>,
    ) {
        prop_assume!(account_a != account_b);
        prop_assume!(data_a != data_b);

        let signature_a = account_a.sign(&data_a);
        let signature_b = account_b.sign(&data_b);

        prop_assert!(account_a.verify(&data_a, &signature_a).is_ok());
        prop_assert!(account_b.verify(&data_b, &signature_b).is_ok());
        prop_assert!(account_b.verify(&data_a, &signature_a).is_err());
        prop_assert!(account_a.verify(&data_b, &signature_b).is_err());
        prop_assert!(account_a.verify(&data_a, &signature_b).is_err());
        prop_assert!(account_a.verify(&data_b, &signature_a).is_err());
        prop_assert!(account_b.verify(&data_a, &signature_b).is_err());
        prop_assert!(account_b.verify(&data_b, &signature_a).is_err());
    }
}
