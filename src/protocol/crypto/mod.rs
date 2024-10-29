pub use curve25519_dalek::{RistrettoPoint as DalekPoint, Scalar as DalekScalar};

mod native;

pub use native::{NativePoint, NativeScalar};

use crate::{protocol::*, Error};

pub trait BlindDiffieHellmanKeyExchange {
    fn hash_to_curve(&self, data: impl EncodeFields) -> Result<Hash, Error>;
    fn blind(&self, data: impl EncodeFields, r: Hash) -> Result<BlindedValue, Error>;
    fn unblind(
        &self,
        public_key: PublicKey,
        blinded_signature: BlindSignature,
        r: SecretKey,
    ) -> Result<Signature, Error>;
    fn sign_blinded(
        &self,
        sk: SecretKey,
        blinded_message: BlindedValue,
    ) -> Result<BlindSignature, Error>;
    fn verify(
        &self,
        pk: PublicKey,
        data: impl EncodeFields,
        signature: Signature,
    ) -> Result<bool, Error>;
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn scalar() -> impl Strategy<Value = Hash> {
        any::<Hash>()
            .prop_map(|x| {
                let mut bytes = x.as_bytes();
                bytes[31] = 0;

                bytes
            })
            .prop_map(|x| Hash::from_slice(&x).unwrap())
    }

    macro_rules! generate_bdhke_tests {
        ($type:ty) => {
            paste::paste! {
                #[::test_strategy::proptest]
                fn [<test_ $type:snake _bdhke_hash_to_curve>](note: Note) {
                    use $crate::protocol::crypto::BlindDiffieHellmanKeyExchange;

                    let bdhke = <$type>::default();
                    let result = bdhke.hash_to_curve(note.clone())?;

                    prop_assert_eq!(result, note.hash());
                }

                #[::test_strategy::proptest]
                fn [<test_ $type:snake _bdhke_full_process>](
                    note: Note,
                    #[strategy(scalar())] r: Hash,
                    sk: SecretKey
                ) {
                    use $crate::protocol::crypto::BlindDiffieHellmanKeyExchange;

                    let bdhke = <$type>::default();

                    let blinded = bdhke.blind(note.clone(), r.clone())?;
                    let blind_signature = bdhke.sign_blinded(sk.clone(), blinded)?;
                    let public_key = sk.public();
                    let signature = bdhke.unblind(public_key, blind_signature, r.into())?;

                    prop_assert_eq!(bdhke.verify(public_key, note, signature), Ok(true));
                }
            }
        };
    }

    type Native = super::native::NativeBdhke;
    generate_bdhke_tests!(Native);
}
