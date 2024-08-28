use mugraph_core::types::*;

#[derive(Debug)]
pub struct User {
    pub notes: Vec<Note>,
}

impl User {
    pub fn new() -> Self {
        Self { ..Self::default() }
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            notes: Default::default(),
        }
    }
}
