use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT as G;
pub use curve25519_dalek::{RistrettoPoint, Scalar};

use crate::{protocol::*, Error};

#[derive(Default)]
pub struct BlindDiffieHellmanKeyExchange;

impl BlindDiffieHellmanKeyExchange {
    #[inline]
    fn hash_to_curve(&self, data: impl AsRef<[u8]>) -> Result<Hash, Error> {
        let data: Scalar = todo!();
        Ok((data * G).into())
    }

    #[inline]
    fn blind(&self, data: impl AsRef<[u8]>, r: Hash) -> Result<BlindedValue, Error> {
        let y: RistrettoPoint = Scalar::try_from(self.hash_to_curve(data)?)? * G;
        let r_scalar: Scalar = r.try_into()?;

        Ok((y + (r_scalar * G)).into())
    }

    #[inline]
    fn unblind(
        &self,
        public_key: PublicKey,
        blinded_signature: BlindSignature,
        r: SecretKey,
    ) -> Result<Signature, Error> {
        let c_prime: RistrettoPoint = blinded_signature.try_into()?;
        let r_scalar: Scalar = r.try_into()?;
        let a_point: RistrettoPoint = public_key.try_into()?;
        let c = c_prime - (r_scalar * a_point);

        Ok(c.into())
    }

    #[inline]
    fn sign_blinded(
        &self,
        sk: SecretKey,
        blinded_message: BlindedValue,
    ) -> Result<BlindSignature, Error> {
        let a: Scalar = sk.try_into()?;
        let b_prime: RistrettoPoint = blinded_message.try_into()?;
        let c_prime = a * b_prime;
        Ok(c_prime.into())
    }

    fn verify(
        &self,
        pk: PublicKey,
        data: impl AsRef<[u8]>,
        signature: Signature,
    ) -> Result<bool, Error> {
        let y: Scalar = self.hash_to_curve(data)?.try_into()?;
        let a_point: RistrettoPoint = pk.try_into()?;
        let c: RistrettoPoint = signature.try_into()?;

        Ok(c == a_point * y)
    }
}
