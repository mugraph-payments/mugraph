use mugraph_core::types::*;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Split(Transaction),
    Join(Transaction),
}
