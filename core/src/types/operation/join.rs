use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "std", derive(test_strategy::Arbitrary))]
pub struct Join {
    pub inputs: [Note; 2],
}

impl SerializeBytes for Join {
    const SIZE: usize = 2 * Note::SIZE;

    fn to_slice(&self, out: &mut [u8]) {
        self.inputs[0].to_slice(&mut out[..Note::SIZE]);
        self.inputs[1].to_slice(&mut out[Note::SIZE..]);
    }

    fn from_slice(bytes: &[u8]) -> Result<Self> {
        let (a, b) = <(Note, Note)>::from_slice(bytes)?;
        Ok(Self { inputs: [a, b] })
    }
}
