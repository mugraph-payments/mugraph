use indexmap::IndexSet;

use crate::{
    error::{Error, Result},
    types::{Asset, AssetName, Atom, Hash, Note, PolicyId, Refresh},
    utils::BitSet32,
};

#[derive(Default)]
pub struct RefreshBuilder {
    pub inputs: Vec<Note>,
    pre_balances: Vec<u128>,
    post_balances: Vec<u128>,
    assets: IndexSet<Asset>,
    outputs: Vec<(u32, u64)>,
}

impl RefreshBuilder {
    pub fn new() -> Self {
        Self {
            pre_balances: vec![0; 8],
            post_balances: vec![0; 8],
            ..Default::default()
        }
    }

    pub fn input(mut self, note: Note) -> Self {
        let asset = Asset {
            policy_id: note.policy_id,
            asset_name: note.asset_name,
        };
        match self.assets.get_index_of(&asset) {
            Some(i) => {
                self.pre_balances[i] += note.amount as u128;
            }
            None => {
                self.pre_balances[self.assets.len()] += note.amount as u128;
                self.assets.insert(asset);
            }
        }

        self.inputs.push(note);

        self
    }

    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    pub fn output(mut self, policy_id: PolicyId, asset_name: AssetName, amount: u64) -> Self {
        let asset = Asset {
            policy_id,
            asset_name,
        };
        match self.assets.get_index_of(&asset) {
            Some(i) => {
                self.post_balances[i] += amount as u128;
                self.outputs.push((i as u32, amount));
            }
            None => {
                let index = self.assets.len();
                self.post_balances[index] += amount as u128;
                self.assets.insert(asset);
                self.outputs.push((index as u32, amount));
            }
        }

        self
    }

    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }

    pub fn build(self) -> Result<Refresh> {
        let mut atoms = Vec::new();
        let mut signatures = Vec::new();
        let mut input_mask = BitSet32::new();
        let delegate = self.inputs[0].delegate;

        for (index, note) in self.inputs.into_iter().enumerate() {
            input_mask.insert(index as u32);

            let asset_id = match self.assets.get_index_of(&Asset {
                policy_id: note.policy_id,
                asset_name: note.asset_name,
            }) {
                Some(a) => a as u32,
                None => {
                    return Err(Error::InvalidOperation {
                        reason: "Missing asset_id for iput".to_string(),
                    });
                }
            };

            atoms.push(Atom {
                delegate: note.delegate,
                asset_id,
                amount: note.amount,
                nonce: note.nonce,
                signature: Some(signatures.len() as u32),
            });

            signatures.push(note.signature);
        }

        for (asset_id, amount) in self.outputs.into_iter() {
            atoms.push(Atom {
                delegate,
                asset_id,
                amount,
                nonce: Hash::random(&mut rand::rng()), // Generate random nonce
                signature: None,
            });
        }

        let refresh = Refresh {
            input_mask,
            atoms,
            asset_ids: self.assets.into_iter().collect(),
            signatures,
        };

        refresh.verify()?;

        Ok(refresh)
    }
}
