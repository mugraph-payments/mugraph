use mugraph_core::types::*;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Refresh(Refresh),
    DoubleRefresh(Refresh),
}
