use crate::types::Note;

#[derive(Default)]
pub struct Wallet {
    pub notes: Vec<Note>,
}

impl Wallet {
    pub fn new() -> Self {
        Self::default()
    }
}
