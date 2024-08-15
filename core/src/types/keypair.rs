use super::{PublicKey, SecretKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Keypair {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl Keypair {
    #[cfg(feature = "std")]
    pub fn random<R: rand::RngCore + rand::CryptoRng>(rng: &mut R) -> Self {
        let secret_key = SecretKey::random(rng);
        let public_key = secret_key.public();

        Self {
            public_key,
            secret_key,
        }
    }
}

#[cfg(feature = "proptest")]
impl proptest::arbitrary::Arbitrary for Keypair {
    type Parameters = ();
    type Strategy = proptest::strategy::BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        use proptest::prelude::*;

        any::<crate::types::SecretKey>()
            .prop_map(|sk| Self {
                public_key: sk.public(),
                secret_key: sk,
            })
            .boxed()
    }
}
