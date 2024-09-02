use mugraph_core::types::*;

pub enum Action {
    Split(Transaction),
    Join(Transaction),
}
