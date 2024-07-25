use crate::{
    crypto::{commitment::TransactionCommitment, dh::DLEQProof, *},
    Hash,
};

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(test_strategy::Arbitrary))]
pub struct Input {
    /// The nullifier from the note, used to prevent double spend.
    ///
    /// Corresponds to x in the protocol.
    #[cfg_attr(test, strategy(crate::testing::point()))]
    pub nullifier: RistrettoPoint,
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
    pub blinded_secret: RistrettoPoint,
}

/// An atom for a swap, the input that is used to generate the RangeProof for
/// the transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(test_strategy::Arbitrary))]
pub struct Atom {
    pub asset_id: Hash,
    #[cfg_attr(test, strategy(1u128..=u128::MAX))]
    pub amount: u128,
    #[cfg_attr(test, strategy(crate::testing::point()))]
    pub nullifier: RistrettoPoint,
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(test_strategy::Arbitrary))]
pub struct Swap {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub commitment: TransactionCommitment,

    /// The unblided signatures for each included Input
    pub witnesses: Vec<Signature>,
}
