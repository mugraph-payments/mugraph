use minicbor::{Decode, Encode, Encoder};
use risc0_zkvm::sha::{Impl, Sha256};
use serde::{Deserialize, Serialize};

use crate::{error::Result, types::*};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub enum Operation {
    #[n(0)]
    #[allow(non_camel_case_types)]
    /// This is a special operation to create notes that we use for testing while commits and decommits are not implemented.
    ///
    /// Please don't use it.
    UNSAFE_Mint {
        #[n(1)]
        output: Sealed<Note>,
    },
    #[n(1)]
    #[cfg_attr(feature = "proptest", weight(3))]
    Consume {
        #[n(0)]
        input: Sealed<Note>,
        #[n(1)]
        output: Note,
    },
    #[n(2)]
    Split {
        #[n(0)]
        input: Sealed<Note>,
        #[b(1)]
        outputs: Vec<Note>,
    },
    #[n(3)]
    Join {
        #[b(0)]
        inputs: Vec<Sealed<Note>>,
        #[n(1)]
        output: Note,
    },
}

impl Operation {
    pub fn id(&self) -> Result<Hash> {
        Ok((*Impl::hash_bytes(&self.to_bytes()?)).into())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        let mut encoder = Encoder::new(&mut buf);

        encoder.encode(&self)?;

        Ok(buf)
    }
}
