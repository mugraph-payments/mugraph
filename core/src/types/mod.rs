#[cfg(test)]
use crate::testing::*;
#[cfg(test)]
use proptest::{collection::vec, prelude::*};
#[cfg(test)]
use test_strategy::Arbitrary;

pub mod delegate;

pub use curve25519_dalek::traits::*;

pub type Hash = [u8; 32];
pub type Point = curve25519_dalek::ristretto::RistrettoPoint;
pub type Scalar = curve25519_dalek::scalar::Scalar;
pub type PublicKey = Point;
pub type SecretKey = Scalar;
pub type CompressedPoint = curve25519_dalek::ristretto::CompressedRistretto;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Signature {
    #[cfg_attr(test, strategy(point()))]
    pub r: Point,
    #[cfg_attr(test, strategy(scalar()))]
    pub s: Scalar,
}

#[derive(Debug)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Note {
    /// The ID for the asset in the Cardano blockchain
    pub asset_id: Hash,

    /// The amount included in this note
    pub amount: u128,

    /// Unblinded signature from the server from this note creation
    ///
    /// Equivalent to C in the protocol, returned by the server after minting or swapping
    /// assets.
    pub signature: Signature,
}

#[derive(Debug)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct UnblindedNote {
    pub asset_id: Hash,
    pub amount: u128,
    pub nonce: Hash,
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Proof;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Commit {}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Swap {
    #[cfg_attr(test, strategy(vec(any::<Signature>(), 0..=16)))]
    pub inputs: Vec<Signature>,

    /// The blinded secrets to be signed by the delegate.
    ///
    /// Corresponds to B' in the protocol.
    #[cfg_attr(test, strategy(vec(point(), 0..=16)))]
    pub outputs: Vec<Point>,

    pub proof: Proof,
}
