pub use curve25519_dalek::{RistrettoPoint as DalekPoint, Scalar as DalekScalar};

mod native;

pub use native::{NativePoint, NativeScalar};

use crate::{protocol::*, Error};

pub trait BlindDiffieHellmanKeyExchange {
    fn hash_to_curve(&self, value: &[u8]) -> Result<Hash, Error>;
    fn blind(&self, value: &[u8], r: SecretKey) -> Result<BlindedValue, Error>;
    fn unblind(&self, blinded_signature: BlindSignature, r: SecretKey) -> Result<Signature, Error>;
    fn sign_blinded(
        &self,
        sk: SecretKey,
        blinded_message: BlindedValue,
    ) -> Result<BlindSignature, Error>;
    fn verify(&self, pk: PublicKey, message: &[u8], signature: Signature) -> Result<bool, Error>;
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    macro_rules! generate_bdhke_tests {
        ($type:ty) => {
            paste::paste! {
                #[::test_strategy::proptest]
                fn [<test_ $type:snake _bdhke_hash_to_curve>](message: Note) {
                    use $crate::protocol::crypto::BlindDiffieHellmanKeyExchange;

                    let bdhke = <$type>::default();
                    let result = bdhke.hash_to_curve(message.as_bytes());
                    prop_assert_eq!(result.is_ok());
                }
            }
        };
    }

    type Native = super::native::NativeBdhke;
    generate_bdhke_tests!(Native);
}
