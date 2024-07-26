use crate::crypto::dh::DLEQProof;
use crate::types::*;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(test_strategy::Arbitrary))]
pub struct Commitment;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(test_strategy::Arbitrary))]
pub struct Input {
    /// The nullifier from the note, used to prevent double spend.
    ///
    /// Corresponds to x in the protocol.
    #[cfg_attr(test, strategy(crate::testing::point()))]
    pub nullifier: Point,

    /// A proof sent by the delegate that the input blinded transaction was
    /// generated correctly.
    pub dleq_proof: DLEQProof,
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(test_strategy::Arbitrary))]
pub struct Output {
    /// The blinded secret to be signed by the delegator.
    ///
    /// Corresponds to B' in the protocol.
    #[cfg_attr(test, strategy(crate::testing::point()))]
    pub blinded_secret: Point,
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(test_strategy::Arbitrary))]
pub struct Swap {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub commitment: Commitment,

    /// The unblided signatures for each included Input
    pub witnesses: Vec<Signature>,
}
