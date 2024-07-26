pub mod delegate;
pub mod note;
pub mod request;

pub use curve25519_dalek::traits::*;

pub type Hash = [u8; 32];
pub type Point = curve25519_dalek::ristretto::RistrettoPoint;
pub type Scalar = curve25519_dalek::scalar::Scalar;
pub type PublicKey = Point;
pub type SecretKey = Scalar;
pub type CompressedPoint = curve25519_dalek::ristretto::CompressedRistretto;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(test_strategy::Arbitrary))]
pub struct Signature {
    #[cfg_attr(test, strategy(crate::testing::point()))]
    pub r: Point,
    #[cfg_attr(test, strategy(crate::testing::scalar()))]
    pub s: Scalar,
}
