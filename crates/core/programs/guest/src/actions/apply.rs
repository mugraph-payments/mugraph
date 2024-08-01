use std::collections::{BTreeMap, BTreeSet};

use mugraph_core::{prover::Claim, types::Note};

pub struct Params {
    pub inputs: BTreeSet<Note>,
    pub outputs: BTreeSet<Note>,
}

pub fn apply(params: Params) -> Claim<u8, u8> {
    let pre_balances = params.inputs.iter().fold(BTreeMap::new(), |mut acc, n| {
        acc.entry(n.asset_id())
            .and_modify(|x| *x += n.amount)
            .or_insert(n.amount);
        acc
    });

    let post_balances = params.outputs.iter().fold(BTreeMap::new(), |mut acc, n| {
        acc.entry(n.asset_id())
            .and_modify(|x| *x += n.amount)
            .or_insert(n.amount);
        acc
    });

    let secrets = params
        .inputs
        .iter()
        .chain(params.outputs.iter())
        .map(|x| x.secret)
        .collect::<BTreeSet<_>>();

    assert_eq!(pre_balances, post_balances);
    assert_eq!(secrets.len(), params.inputs.len() + params.outputs.len());

    // TODO: Verify input proofs

    todo!()
}
