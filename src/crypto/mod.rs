use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT as G;
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

#[inline]
pub fn secret_to_public(key: SecretKey) -> Result<PublicKey, Error> {
    let key: curve25519_dalek::Scalar = key.try_into()?;
    let res = key * G;

    Ok(res.into())
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::BlindDiffieHellmanKeyExchange;
    use crate::{crypto::*, protocol::circuit::*, Error};

    type Result = std::result::Result<(), TestCaseError>;

    fn test_htc<T: BlindDiffieHellmanKeyExchange>(note: Note, bdhke: T) -> Result {
        let result = bdhke.hash_to_curve(note.clone())?;

        let mut builder = circuit_builder();
        let inputs = builder.add_virtual_targets(13);
        let targets = hash_to_curve(&mut builder, &inputs);

        builder.register_public_inputs(&[inputs.clone(), targets.elements.to_vec()].concat());

        let circuit = builder.build::<C>();

        let mut pw = PartialWitness::new();
        pw.set_target_arr(&inputs, &note.as_fields_with_prefix());
        let proof = circuit.prove(pw).map_err(Error::from)?;

        prop_assert_eq!(Hash::from_fields(&proof.public_inputs[13..])?, result);
        prop_assert!(circuit.verify(proof).is_ok());

        Ok(())
    }

    fn test_blind<T: BlindDiffieHellmanKeyExchange>(
        note: Note,
        secret_key: SecretKey,
        bdhke: T,
        r: Hash,
    ) -> Result {
        let blinded = bdhke.blind(note.clone(), r)?;
        let public_key = secret_to_public(secret_key)?;
        let blind_signature = bdhke.sign_blinded(secret_key, blinded)?;
        let signature = bdhke.unblind(public_key, blind_signature, r)?;

        prop_assert_eq!(bdhke.verify(public_key, note, signature), Ok(true));

        Ok(())
    }

    macro_rules! generate_bdhke_tests {
        ($type:ty) => {
            paste::paste! {
                #[::test_strategy::proptest]
                fn [<test_ $type:snake _bdhke_hash_to_curve>](note: Note) {
                    test_htc(note, <$type>::default())?;
                }

                #[::test_strategy::proptest]
                fn [<test_ $type:snake _bdhke_full_process>]( note: Note, r: Hash, secret_key: SecretKey) {
                    test_blind(note, secret_key, <$type>::default(), r)?;
                }
            }
        };
    }

    type Native = super::native::NativeBdhke;
    generate_bdhke_tests!(Native);
}
