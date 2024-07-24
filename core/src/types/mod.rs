use crate::{
    crypto::{diffie_hellman::DLEQProof, RistrettoPoint, Scalar},
    Hash,
};

pub struct Proof {
    pub asset_id: Hash,
    pub amount_commitment: RistrettoPoint,
    pub secret: Scalar,
    pub unblinded_signature: RistrettoPoint,
}

pub struct BlindedMessage {
    pub asset_id: Hash,
    pub amount_commitment: RistrettoPoint,
    pub blinded_point: RistrettoPoint,
}

pub struct BlindSignature {
    pub signed_point: RistrettoPoint,
    pub dleq_proof: DLEQProof,
}

pub struct SchnorrSignature {
    pub r: RistrettoPoint,
    pub s: Scalar,
}

pub struct TransactionSnapshot {
    pub transaction_id: Hash,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub timestamp: u64,
}

pub struct TransactionInput {
    pub asset_id: Hash,
    pub amount: u64,
    pub proof: Proof,
}

pub struct TransactionOutput {
    pub asset_id: Hash,
    pub amount: u64,
    pub blinded_message: BlindedMessage,
}

pub struct DelegatorInfo {
    pub active_keys: Vec<RistrettoPoint>,
    pub expired_keys: Vec<RistrettoPoint>,
    pub generator: RistrettoPoint,
}
