use crate::{Error, Hash, Result, SerializeBytes};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Fission {
    pub a: Hash,
    pub b: Hash,
    pub c: Hash,
}

impl SerializeBytes for Fission {
    const SIZE: usize = 3 * 32;

    fn to_slice(&self, out: &mut [u8]) {
        out[..32].copy_from_slice(&*self.a);
        out[32..64].copy_from_slice(&*self.b);
        out[64..].copy_from_slice(&*self.c);
    }

    fn from_slice(input: &[u8]) -> Result<Self> {
        if input.len() < Self::SIZE {
            return Err(Error::FailedDeserialization);
        }

        let a = input[..32].try_into()?;
        let b = input[Hash::SIZE..Hash::SIZE * 2].try_into()?;
        let c = input[Hash::SIZE..Hash::SIZE * 2].try_into()?;

        Ok(Self { a, b, c })
    }
}
