pub use curve25519_dalek::{RistrettoPoint as NativePoint, Scalar as NativeScalar};

use super::BlindDiffieHellmanKeyExchange;

#[derive(Default)]
pub struct NativeBdhke;

impl BlindDiffieHellmanKeyExchange for NativeBdhke {
    fn hash_to_curve(&self, value: &[u8]) -> Result<super::Hash, crate::Error> {
        todo!()
    }

    fn blind(&self, value: &[u8], r: super::SecretKey) -> Result<super::BlindedValue, crate::Error> {
        todo!()
    }

    fn unblind(
        &self,
        _blinded_signature: super::BlindSignature,
        _r: super::SecretKey,
    ) -> Result<super::Signature, crate::Error> {
        todo!()
    }

    fn sign_blinded(
        &self,
        _sk: super::SecretKey,
        _blinded_message: super::BlindedValue,
    ) -> Result<super::BlindSignature, crate::Error> {
        todo!()
    }

    fn verify(
        &self,
        _pk: super::PublicKey,
        _message: &[u8],
        _signature: super::Signature,
    ) -> Result<bool, crate::Error> {
        todo!()
    }
}
