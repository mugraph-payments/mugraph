use crate::{Note, Result};
use serde::{Deserialize, Serialize};

mod join;
mod split;

pub use self::{join::*, split::*};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Join {
    pub inputs: [Note; 2],
}

impl Join {
    pub const SIZE: usize = 2 * Note::SIZE;

    pub fn to_slice(&self, out: &mut [u8; Self::SIZE]) {
        out[..Note::SIZE].copy_from_slice(&self.inputs[0].as_bytes());
        out[Note::SIZE..2 * Note::SIZE].copy_from_slice(&self.inputs[1].as_bytes());
    }

    pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Result<Self> {
        let inputs = [
            Note::from_bytes(&bytes[..Note::SIZE].try_into().unwrap())?,
            Note::from_bytes(&bytes[Note::SIZE..2 * Note::SIZE].try_into().unwrap())?,
        ];

        Ok(Self { inputs })
    }
}
