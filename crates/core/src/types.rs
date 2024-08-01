pub use std::collections::BTreeSet;

use minicbor::{Decode, Encode};
use serde::{Deserialize, Serialize};

pub type Hash = [u8; 32];
pub type Signature = [u8; 64];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Encode, Decode, Serialize, Deserialize)]
pub enum Version {
    #[n(0)]
    #[serde(rename = "v0")]
    #[default]
    V0,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub struct Proof {
    /// The version of the proof (for upgradeability)
    #[n(0)]
    pub version: Version,
    /// The proof data
    #[cbor(n(1), with = "minicbor::bytes")]
    #[serde(with = "serde_bytes")]
    pub seal: [u8; 256],
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub struct Receipt {
    /// The program that generated this receipt
    #[n(0)]
    pub program_id: Hash,
    /// A list of consumed secrets
    #[n(1)]
    pub inputs: BTreeSet<Hash>,
    /// A list of note commitments
    #[n(2)]
    pub outputs: BTreeSet<Hash>,
    /// A zero-knowledge proof of validity for this receipt
    #[n(3)]
    pub proof: Proof,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
/// A note representing the redemption right of a value on-chain
pub struct Note {
    /// Hash of the receipt that generated this note
    #[n(0)]
    pub parent: Hash,
    /// The on-chain policy_id that this note is associated with
    #[n(1)]
    pub policy_id: Hash,
    /// The asset_name that this note is associated with
    #[n(2)]
    pub asset_name: String,
    /// The amount of the asset that this note represents
    #[n(3)]
    pub amount: u64,
    /// The secret key to spend this note
    #[n(4)]
    pub secret: Hash,
    /// A program that must be ran to spend this Note
    #[n(5)]
    pub script: Option<Script>,
    /// The zero-knowledge proof that generated this note
    #[n(6)]
    pub receipt: Proof,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub struct Script {
    /// The ID of the program to be ran
    #[n(0)]
    pub program_id: Hash,
    /// The ELF data for the program
    #[n(1)]
    pub elf: Vec<u8>,
    /// The state of this script
    #[n(2)]
    pub state: Vec<u8>,
}
