use mugraph_core::types::{Note, Receipt};

pub struct Params {
    pub inputs: Vec<Note>,
    pub outputs: Vec<Note>,
}

pub struct Output {}

pub fn apply(_params: Params) -> Output {
    todo!()
}
