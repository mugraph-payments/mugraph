use mugraph_core::types::*;

#[derive(Debug, Default)]
pub struct User {
    pub notes: Vec<Note>,
}

impl User {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }
}
