use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT as G;
pub use curve25519_dalek::{RistrettoPoint as NativePoint, Scalar as NativeScalar};

use super::*;
use crate::{protocol::Hash, Error};

#[derive(Default)]
pub struct NativeBdhke;

impl BlindDiffieHellmanKeyExchange for NativeBdhke {
    fn hash_to_curve(&self, data: impl EncodeFields) -> Result<Hash, Error> {
        let data: NativeScalar = data.hash().try_into()?;
        Ok((data * G).into())
    }

    fn blind(&self, data: impl EncodeFields, r: Hash) -> Result<BlindedValue, Error> {
        let y: NativePoint = NativeScalar::try_from(self.hash_to_curve(data)?)? * G;
        let r_scalar: NativeScalar = r.try_into()?;

        Ok((y + (r_scalar * G)).into())
    }

    fn unblind(
        &self,
        public_key: PublicKey,
        blinded_signature: BlindSignature,
        r: SecretKey,
    ) -> Result<Signature, Error> {
        let c_prime: NativePoint = blinded_signature.try_into()?;
        let r_scalar: NativeScalar = r.try_into()?;
        let a_point: NativePoint = public_key.try_into()?;
        let c = c_prime - (r_scalar * a_point);
        Ok(c.into())
    }

    fn sign_blinded(
        &self,
        sk: SecretKey,
        blinded_message: BlindedValue,
    ) -> Result<BlindSignature, Error> {
        let a: NativeScalar = sk.try_into()?;
        let b_prime: NativePoint = blinded_message.try_into()?;
        let c_prime = a * b_prime;
        Ok(c_prime.into())
    }

    fn verify(
        &self,
        pk: PublicKey,
        data: impl EncodeFields,
        signature: Signature,
    ) -> Result<bool, Error> {
        let y: NativeScalar = self.hash_to_curve(data)?.try_into()?;
        let a_point: NativePoint = pk.try_into()?;
        let c: NativePoint = signature.try_into()?;

        Ok(c == a_point * y)
    }
}
