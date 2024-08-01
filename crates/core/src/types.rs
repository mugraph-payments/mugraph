pub type Hash = [u8; 32];
pub type Signature = [u8; 64];

pub enum Version {
    V0,
}

pub struct Proof {
    pub version: Version,
    pub seal: [u8; 256],
}

pub struct Receipt {
    pub inputs: Vec<Receipt>,
    pub outputs: Vec<Hash>,
    pub proof: Proof,
}

pub struct Note {
    /// The on-chain policy_id that this note is associated with
    pub policy_id: Hash,
    /// The asset_name that this note is associated with
    pub asset_name: String,
    /// The amount of the asset that this note represents
    pub amount: u64,
    /// The zero-knowledge proof that generated this note
    pub receipt: Receipt,
}
