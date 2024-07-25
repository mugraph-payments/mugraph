use crate::{
    crypto::{commitment::TransactionCommitment, dh::DLEQProof, *},
    Hash,
};

pub struct Input {
    /// The nullifier from the note, used to prevent double spend.
    ///
    /// Corresponds to x in the protocol.
    pub nullifier: RistrettoPoint,
    /// A proof sent by the delegate that the input blinded transaction was
    /// generated correctly.
    pub dleq_proof: DLEQProof,
}

pub struct Output {
    /// The blinded secret to be signed by the delegator.
    ///
    /// Corresponds to B' in the protocol.
    pub blinded_secret: RistrettoPoint,
}

/// An atom for a swap, the input that is used to generate the RangeProof for
/// the transaction.
pub struct Atom {
    pub asset_id: Hash,
    pub amount: u128,
    pub nullifier: RistrettoPoint,
}

pub struct Swap {
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub commitment: TransactionCommitment,

    /// The unblided signatures for each included Input
    pub witnesses: Vec<Signature>,
}
