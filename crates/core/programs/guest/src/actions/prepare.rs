use std::collections::{BTreeMap, BTreeSet};

use mugraph_core::types::Note;

pub struct Params {
    pub inputs: BTreeSet<Note>,
    pub outputs: Vec<Note>,
}

pub struct Output {}

pub fn prepare(params: Params) -> Output {
    let pre_balances = params.inputs.iter().fold(BTreeMap::new(), |mut acc, n| {
        acc.entry(n.asset_id())
            .and_modify(|x| *x += n.amount)
            .or_insert(n.amount);
        acc
    });

    let post_balances = params.outputs.iter().fold(BTreeMap::new(), |mut acc, c| {
        acc.entry(c.asset_id())
            .and_modify(|x| *x += c.amount)
            .or_insert(c.amount);
        acc
    });

    assert_eq!(pre_balances, post_balances);

    todo!()
}
