use std::fmt;

use test_strategy::Arbitrary;

use crate::{
    protocol::*,
    Decode,
    Encode,
    Error,
};
use mucodec::ReprBytes;

#[derive(
    Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Arbitrary
)]
pub struct SealedNote {
    pub issuing_key: PublicKey,
    pub host: String,
    #[strategy(1u16..)]
    pub port: u16,
    pub note: Note,
    pub signature: Signature,
}

impl SealedNote {
    #[inline]
    pub fn host(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Arbitrary)]
pub struct Note {
    #[filter(#asset_id != Hash::zero())]
    pub asset_id: Hash,
    #[filter(#asset_name != Name::zero())]
    pub asset_name: Name,
    #[strategy(1u64..)]
    pub amount: u64,
    #[filter(#nonce != Hash::zero())]
    pub nonce: Hash,
}
impl Note {
    #[inline]
    pub fn asset_name(&self) -> String {
        self.asset_name.to_string()
    }
}

impl fmt::Display for Note {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Debug for Note {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Note")
            .field("asset_id", &self.asset_id)
            .field("asset_name", &self.asset_name())
            .field("amount", &self.amount)
            .field("nonce", &self.nonce)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::*;
    use crate::test_encode_decode;

    test_encode_decode!(Note);
}
