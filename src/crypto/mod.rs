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
    use std::panic::UnwindSafe;

    use proptest::prelude::*;

    use super::BlindDiffieHellmanKeyExchange;
    use crate::{crypto::*, protocol::circuit::*, unwind_panic, Error};

    type Result = std::result::Result<(), TestCaseError>;

    fn test_htc<T: BlindDiffieHellmanKeyExchange + UnwindSafe>(note: Note, bdhke: T) -> Result {
        let fields = note.as_fields();
        let native = bdhke.hash_to_curve(note.clone())?.into();

        let proof = unwind_panic(move || {
            let mut builder = circuit_builder();
            let inputs = builder.add_virtual_targets(Note::FIELD_SIZE);
            let expected = builder.add_virtual_hash();
            let result = hash_to_curve(&mut builder, &inputs);

            builder.connect_hashes(result, expected);

            builder.register_public_inputs(&inputs);
            builder.register_public_inputs(&expected.elements);

            let circuit = builder.build::<C>();

            let mut pw = PartialWitness::new();
            pw.set_target_arr(&inputs, &fields);
            pw.set_hash_target(expected, native);

            (circuit.prove(pw)).map_err(Error::from)
        })?;

        prop_assert_eq!(
            Hash::from_fields(&proof.public_inputs[Note::FIELD_SIZE..])?,
            bdhke.hash_to_curve(note.clone())?
        );

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
                #[::test_strategy::proptest(cases = 1)]
                fn [<test_ $type:snake _hash_to_curve>](note: Note) {
                    test_htc(note, <$type>::default())?;
                }

                #[::test_strategy::proptest]
                #[ignore]
                fn [<test_ $type:snake _bdhke_full_process>]( note: Note, r: Hash, secret_key: SecretKey) {
                    test_blind(note, secret_key, <$type>::default(), r)?;
                }
            }
        };
    }

    type Native = super::native::NativeBdhke;
    generate_bdhke_tests!(Native);
}
