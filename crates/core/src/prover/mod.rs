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
    pub inputs: Vec<Signature>,
    pub outputs: Vec<Hash>,
    pub proof: Proof,
}

pub struct Claim<P, R> {
    pub public: P,
    pub private: R,
}
