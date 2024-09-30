use crate::error::{Error, Result};
use rand::prelude::*;
use rand::{CryptoRng, RngCore};
use rand_chacha::ChaCha20Rng;

impl Seed for ChaCha20Rng {}

use super::{Point, Scalar};

pub trait Pair: Send + Sync + Clone {
    type Public: Public;
    type Secret: Secret;
    type Signature: Signature;
    type Seed: Seed;

    fn sign(&self, seed: &mut Self::Seed, message: &[u8]) -> Self::Signature;
    fn verify(&self, signature: &Self::Signature, message: &[u8]) -> Result<()>;
    fn public(&self) -> Self::Public;
    fn secret(&self) -> Self::Secret;
    fn random(seed: &mut impl Seed) -> Self;
}

pub trait Signature {}

pub trait Seed: RngCore + CryptoRng {}

pub trait Public {
    fn to_point(&self) -> Result<Point>;
}

pub trait Secret {
    fn to_scalar(&self) -> Scalar;
}
