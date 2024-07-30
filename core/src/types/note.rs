use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Note {
    pub asset_id: Hash,
    pub amount: u64,
    pub nullifier: Signature,
}

impl SerializeBytes for Note {
    const SIZE: usize = Hash::SIZE + u64::SIZE + Signature::SIZE;

    fn to_slice(&self, out: &mut [u8]) {
        let mut w = Writer::new(out);

        w.write(&self.asset_id);
        w.write(&self.amount);
        w.write(&self.nullifier);
    }

    fn from_slice(input: &[u8]) -> Result<Self> {
        let mut r = Reader::new(input);

        Ok(Self {
            asset_id: r.read()?,
            amount: r.read()?,
            nullifier: r.read()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct BlindedNote {
    pub asset_id: Hash,
    pub amount: u64,
    pub secret: Hash,
}

impl BlindedNote {
    pub fn unblind(self, signature: Signature) -> Note {
        Note {
            asset_id: self.asset_id,
            amount: self.amount,
            nullifier: signature,
        }
    }
}

impl SerializeBytes for BlindedNote {
    const SIZE: usize = Hash::SIZE + u64::SIZE + Hash::SIZE;

    fn to_slice(&self, out: &mut [u8]) {
        let mut w = Writer::new(out);

        w.write(&self.asset_id);
        w.write(&self.amount);
        w.write(&self.secret);
    }

    fn from_slice(input: &[u8]) -> Result<Self> {
        let mut r = Reader::new(input);

        Ok(Self {
            asset_id: r.read()?,
            amount: r.read()?,
            secret: r.read()?,
        })
    }
}
