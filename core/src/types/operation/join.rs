use crate::{Note, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Join {
    pub inputs: [Note; 2],
}

impl Join {
    pub const SIZE: usize = 2 * Note::SIZE;

    pub fn to_slice(&self, out: &mut [u8]) {
        self.inputs[0].to_slice(&mut out[..Note::SIZE]);
        self.inputs[1].to_slice(&mut out[Note::SIZE..2 * Note::SIZE]);
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let inputs = [
            Note::from_bytes(&bytes[..Note::SIZE])?,
            Note::from_bytes(&bytes[Note::SIZE..2 * Note::SIZE])?,
        ];

        Ok(Self { inputs })
    }
}
