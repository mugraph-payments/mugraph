use crate::{Hash, Result, SerializeBytes};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Fusion {
    pub a: Hash,
    pub b: Hash,
    pub c: Hash,
}

impl SerializeBytes for Fusion {
    const SIZE: usize = 3 * 32;

    fn to_slice(&self, out: &mut [u8]) {
        self.a.to_slice(&mut out[..Hash::SIZE]);
        self.b.to_slice(&mut out[Hash::SIZE..Hash::SIZE * 2]);
        self.c.to_slice(&mut out[Hash::SIZE * 2..Hash::SIZE * 3]);
    }

    fn from_slice(input: &[u8]) -> Result<Self> {
        Ok(Self {
            a: Hash::from_slice(&input[..Hash::SIZE])?,
            b: Hash::from_slice(&input[Hash::SIZE..Hash::SIZE * 2])?,
            c: Hash::from_slice(&input[Hash::SIZE * 2..Hash::SIZE * 3])?,
        })
    }
}
