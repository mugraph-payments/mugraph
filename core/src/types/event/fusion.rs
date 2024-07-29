use crate::{Hash, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Fusion {
    pub a: Hash,
    pub b: Hash,
    pub c: Hash,
}

impl Fusion {
    pub const SIZE: usize = 3 * 32;

    pub fn to_slice(&self, out: &mut [u8; Self::SIZE]) {
        out[..32].copy_from_slice(&*self.a);
        out[32..64].copy_from_slice(&*self.b);
        out[64..].copy_from_slice(&*self.c);
    }

    pub fn from_bytes(bytes: &[u8; Self::SIZE]) -> Result<Self> {
        let mut buf = [0u8; 32];

        buf.copy_from_slice(&bytes[..32]);
        let a = Hash(buf);

        buf.copy_from_slice(&bytes[32..64]);
        let b = Hash(buf);

        buf.copy_from_slice(&bytes[64..]);
        let c = Hash(buf);

        Ok(Self { a, b, c })
    }
}
