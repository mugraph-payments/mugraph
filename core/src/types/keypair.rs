use proptest::prelude::*;

use super::{PublicKey, SecretKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Keypair {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl Keypair {
    pub fn random<R: rand::RngCore + rand::CryptoRng>(rng: &mut R) -> Self {
        let secret_key = SecretKey::random(rng);
        let public_key = secret_key.public();

        Self {
            public_key,
            secret_key,
        }
    }
}

impl Arbitrary for Keypair {
    type Parameters = ();
    type Strategy = proptest::strategy::BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        any::<crate::types::SecretKey>()
            .prop_map(|sk| Self {
                public_key: sk.public(),
                secret_key: sk,
            })
            .boxed()
    }
}
