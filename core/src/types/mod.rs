use crate::{
    crypto::{RistrettoPoint, Scalar},
    Hash,
};

pub struct Proof {
    pub asset_id: Hash,
    /// Pedersen Commitment on the input amount (C_i^in)
    pub amount_commitment: RistrettoPoint,
    pub secret: RistrettoPoint,
    pub unblinded_signature: RistrettoPoint,
}

pub struct BlindedMessage {
    pub asset_id: Hash,
    /// Pedersen Commitment on the output amount (C_i^out)
    pub amount_commitment: RistrettoPoint,
    pub blinded_point: RistrettoPoint,
}

pub struct BlindSignature {
    pub signed_point: RistrettoPoint,
    pub dleq_proof: DLEQProof,
}

pub struct DLEQProof {
    pub e: Scalar,
    pub s: Scalar,
}
