use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Split {
    pub server_key: PublicKey,
    pub input: Note,
    pub amount: u64,
}

impl SerializeBytes for Split {
    const SIZE: usize = 32 + Note::SIZE + 8;

    fn to_slice(&self, out: &mut [u8]) {
        self.server_key.to_slice(&mut out[..32]);
        self.input.to_slice(&mut out[32..32 + Note::SIZE]);
        self.amount.to_slice(&mut out[32 + Note::SIZE..]);
    }

    fn from_slice(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(crate::Error::FailedDeserialization);
        }

        Ok(Self {
            server_key: PublicKey::from_slice(&bytes[..32])?,
            input: Note::from_slice(&bytes[32..32 + Note::SIZE])?,
            amount: u64::from_slice(&bytes[32 + Note::SIZE..])?,
        })
    }
}
