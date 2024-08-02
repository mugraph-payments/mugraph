use std::collections::BTreeSet;

use eyre::Result;
use minicbor::{Decode, Encode, Encoder};
use risc0_zkvm::sha::{Impl, Sha256};
use serde::{Deserialize, Serialize};
mod hash;
mod seal;
mod transaction;

pub use self::{hash::Hash, seal::Sealed, transaction::*};

pub type Signature = [u8; 64];

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
/// A note representing the redemption right of a value on-chain
pub struct Note {
    /// A unique hash for this note
    #[n(0)]
    pub id: Hash,
    /// The ID of the program that generated this note
    #[n(1)]
    pub program_id: Hash,
    /// The on-chain policy_id that this note is associated with
    #[n(2)]
    pub policy_id: Hash,
    /// The asset_name that this note is associated with
    #[n(3)]
    pub asset_name: String,
    /// The amount of the asset that this note represents
    #[n(4)]
    pub amount: u64,
    /// A program that must be ran to spend this Note
    #[n(5)]
    pub script: Option<Script>,
}

impl Note {
    pub fn asset_id(&self) -> Hash {
        let mut buf = Vec::new();
        buf[..32].copy_from_slice(self.policy_id.as_ref());
        buf[32..(self.policy_id.len() * 8)].copy_from_slice(self.asset_name.as_bytes());

        (*Impl::hash_bytes(&buf)).into()
    }

    pub fn nullifier(&self) -> Result<Hash> {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder
            .begin_bytes()?
            .bytes(self.id.as_ref())?
            .bytes(self.policy_id.as_ref())?
            .bytes(self.asset_name.as_bytes())?
            .bytes(&self.amount.to_le_bytes())?
            .bytes(
                self.script
                    .as_ref()
                    .map(|s| s.program_id)
                    .unwrap_or_default()
                    .as_ref(),
            )?
            .end()?;

        Ok((*Impl::hash_bytes(&buf)).into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
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

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
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
}
