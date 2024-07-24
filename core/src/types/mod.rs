use crate::{crypto::RistrettoPoint, Hash};

pub struct Input {
    pub asset_id: Hash,
    /// Pedersen Commitment on the output amount
    pub amount: RistrettoPoint,
    pub secret: RistrettoPoint,
    pub unblinded_signature: RistrettoPoint,
}

pub struct Output {
    pub asset_id: Hash,
    pub amount: u128,
    pub signature: RistrettoPoint,
}
