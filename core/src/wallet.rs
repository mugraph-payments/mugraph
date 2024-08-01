use crate::types::Note;

#[derive(Default, Debug)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
pub struct Wallet {
    pub notes: Vec<Note>,
}

impl Wallet {
    pub fn new() -> Self {
        Self::default()
    }
}
