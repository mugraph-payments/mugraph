use crate::crypto::*;

pub struct Input {
    /// The nullifier from the note
    pub nullifier: RistrettoPoint,
    /// The bulletproof for the pedersen commitment on the note amount
    pub commitment: RistrettoPoint,
}

pub struct Output {
    /// The blinded secret to be signed by the delegator.
    ///
    /// Corresponds to B' in the protocol.
    pub blinded_secret: RistrettoPoint,
}

/// The Range proof to verify the swap operation has the correct values.
///
/// It is split into two smaller proofs as Bulletproofs is u64 while the amounts are u128.
pub struct Proof {
    pub high: RangeProof,
    pub low: RangeProof,
}

pub struct Swap {
    pub inputs: Vec<RistrettoPoint>,
    pub outputs: Vec<RistrettoPoint>,
    pub proof: Proof,
    /// The unblided signatures for each input included in the Swap.
    pub witnesses: Vec<Signature>,
}
