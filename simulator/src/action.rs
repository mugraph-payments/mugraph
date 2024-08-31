use mugraph_core::types::*;

pub enum Action {
    Transfer {
        from: u32,
        to: u32,
        asset_id: Hash,
        amount: u64,
    },
}
