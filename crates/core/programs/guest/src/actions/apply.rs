use mugraph_core::types::Note;

pub struct Params {
    pub inputs: Vec<Note>,
    pub outputs: Vec<Note>,
}

pub fn apply(_input: u8) -> Claim<u8, u8> {
    // 1. Verify inputs and outputs have same combination of assets
    // 2. Verify inputs and outputs have same amount per asset
    // 3. Verify input contains data for all input proofs
    todo!()
}
