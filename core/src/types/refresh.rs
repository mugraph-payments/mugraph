use serde::{Deserialize, Serialize};

use super::{COMMITMENT_INPUT_SIZE, PublicKey};
use crate::{
    error::Error,
    types::{ASSET_ID_BYTES_SIZE, Asset, Hash, Signature, write_asset_bytes},
    utils::BitSet32,
};

pub const MAX_ATOMS: usize = 12;
pub const MAX_INPUTS: usize = 4;
pub const MAX_OUTPUTS: usize = 8;
pub const DATA_SIZE: usize = 256 * MAX_ATOMS;

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    std::hash::Hash,
    test_strategy::Arbitrary,
    PartialOrd,
    Ord,
)]
pub struct Atom {
    pub delegate: PublicKey,
    pub asset_id: u32,
    pub amount: u64,
    pub nonce: Hash,
    pub signature: Option<u32>,
}

impl Atom {
    pub fn commitment(&self, assets: &[Asset]) -> Hash {
        let mut output = [0u8; COMMITMENT_INPUT_SIZE];

        output[0..32].copy_from_slice(self.delegate.as_ref());
        let mut asset_bytes = [0u8; ASSET_ID_BYTES_SIZE];
        let asset = &assets[self.asset_id as usize];
        write_asset_bytes(&asset.policy_id, &asset.asset_name, &mut asset_bytes);
        output[32..96].copy_from_slice(&asset_bytes);
        output[96..104].copy_from_slice(&self.amount.to_le_bytes());
        output[104..136].copy_from_slice(self.nonce.as_ref());

        Hash::digest(&output)
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    std::hash::Hash,
    test_strategy::Arbitrary,
    PartialOrd,
    Ord,
)]
pub struct Refresh {
    #[serde(rename = "m")]
    pub input_mask: BitSet32,
    #[serde(rename = "a")]
    pub atoms: Vec<Atom>,
    #[serde(rename = "a_")]
    pub asset_ids: Vec<Asset>,
    #[serde(rename = "s")]
    pub signatures: Vec<Signature>,
}

impl Refresh {
    pub fn is_input(&self, id: usize) -> bool {
        self.input_mask.contains(id as u32)
    }

    pub fn is_output(&self, id: usize) -> bool {
        !self.input_mask.contains(id as u32)
    }

    pub fn verify(&self) -> Result<(), Error> {
        let mut pre = vec![0; self.asset_ids.len()];
        let mut post = vec![0; self.asset_ids.len()];

        for (i, atom) in self.atoms.iter().enumerate() {
            let target = match self.is_input(i) {
                true => &mut pre,
                false => &mut post,
            };

            match self.asset_ids.get(atom.asset_id as usize) {
                Some(_) => {}
                None => {
                    return Err(Error::InvalidOperation {
                        reason: "Asset ids are not valid".to_string(),
                    });
                }
            }

            target[atom.asset_id as usize] += atom.amount as u128;
        }

        if pre != post {
            return Err(Error::InvalidOperation {
                reason: format!(
                    "unbalanced transaction, expected {} units got {} units",
                    pre.iter().sum::<u128>(),
                    post.iter().sum::<u128>()
                ),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::*;
    /// Strategy that generates a structurally valid, balanced Refresh.
    fn balanced_refresh() -> impl Strategy<Value = Refresh> {
        (
            any::<PublicKey>(),
            any::<Asset>(),
            1u64..=500_000,
            proptest::collection::vec(1u64..=500_000, 1..=4),
        )
            .prop_map(|(delegate, asset, input_amount, split_weights)| {
                let total_weight: u64 = split_weights.iter().sum();
                let mut output_amounts: Vec<u64> = split_weights
                    .iter()
                    .map(|w| input_amount * w / total_weight)
                    .collect();

                // Distribute remainder to first output to guarantee exact balance
                let output_sum: u64 = output_amounts.iter().sum();
                if output_sum < input_amount {
                    output_amounts[0] += input_amount - output_sum;
                }

                let mut input_mask = BitSet32::new();
                input_mask.insert(0);

                let mut atoms = vec![Atom {
                    delegate,
                    asset_id: 0,
                    amount: input_amount,
                    nonce: Hash::default(),
                    signature: Some(0),
                }];

                for amount in &output_amounts {
                    atoms.push(Atom {
                        delegate,
                        asset_id: 0,
                        amount: *amount,
                        nonce: Hash::default(),
                        signature: None,
                    });
                }

                Refresh {
                    input_mask,
                    atoms,
                    asset_ids: vec![asset],
                    signatures: vec![Signature::default()],
                }
            })
    }

    #[proptest]
    fn prop_balanced_refresh_verifies(#[strategy(balanced_refresh())] refresh: Refresh) {
        prop_assert!(refresh.verify().is_ok());
    }

    #[proptest]
    fn prop_unbalanced_refresh_fails(#[strategy(balanced_refresh())] mut refresh: Refresh) {
        // Find first output atom index
        let output_idx = refresh
            .atoms
            .iter()
            .enumerate()
            .position(|(i, _)| refresh.is_output(i));

        if let Some(idx) = output_idx {
            refresh.atoms[idx].amount = refresh.atoms[idx].amount.saturating_add(1);
            prop_assert!(refresh.verify().is_err());
        }
    }

    /// Strategy that generates a balanced multi-asset Refresh (2 assets).
    ///
    /// Each asset has its own input and outputs that sum correctly.
    /// This exercises the per-asset balance check that single-asset tests miss.
    fn multi_asset_refresh() -> impl Strategy<Value = Refresh> {
        (
            any::<PublicKey>(),
            any::<Asset>(),
            any::<Asset>(),
            1u64..=500_000,
            1u64..=500_000,
            proptest::collection::vec(1u64..=500_000, 1..=3),
            proptest::collection::vec(1u64..=500_000, 1..=3),
        )
            .prop_filter("assets must differ", |(_d, a, b, ..)| a != b)
            .prop_map(
                |(delegate, asset_a, asset_b, amount_a, amount_b, weights_a, weights_b)| {
                    fn split(amount: u64, weights: &[u64]) -> Vec<u64> {
                        let total_w: u64 = weights.iter().sum();
                        let mut out: Vec<u64> =
                            weights.iter().map(|w| amount * w / total_w).collect();
                        let sum: u64 = out.iter().sum();
                        if sum < amount {
                            out[0] += amount - sum;
                        }
                        out
                    }

                    let outputs_a = split(amount_a, &weights_a);
                    let outputs_b = split(amount_b, &weights_b);

                    let mut input_mask = BitSet32::new();
                    input_mask.insert(0);
                    input_mask.insert(1);

                    let mut atoms = vec![
                        Atom {
                            delegate,
                            asset_id: 0,
                            amount: amount_a,
                            nonce: Hash::default(),
                            signature: Some(0),
                        },
                        Atom {
                            delegate,
                            asset_id: 1,
                            amount: amount_b,
                            nonce: Hash::default(),
                            signature: Some(1),
                        },
                    ];

                    for &amt in &outputs_a {
                        atoms.push(Atom {
                            delegate,
                            asset_id: 0,
                            amount: amt,
                            nonce: Hash::default(),
                            signature: None,
                        });
                    }
                    for &amt in &outputs_b {
                        atoms.push(Atom {
                            delegate,
                            asset_id: 1,
                            amount: amt,
                            nonce: Hash::default(),
                            signature: None,
                        });
                    }

                    Refresh {
                        input_mask,
                        atoms,
                        asset_ids: vec![asset_a, asset_b],
                        signatures: vec![Signature::default(), Signature::default()],
                    }
                },
            )
    }

    /// Multi-asset balanced refresh must verify.
    ///
    /// Validates the per-asset balance check path with 2 independent assets.
    #[proptest]
    fn prop_multi_asset_balanced_verifies(#[strategy(multi_asset_refresh())] refresh: Refresh) {
        prop_assert!(refresh.verify().is_ok());
    }

    /// Multi-asset: moving value from asset A to asset B must fail.
    ///
    /// Metamorphic: a balanced multi-asset refresh becomes unbalanced when
    /// we shift 1 unit from one asset's output to the other. This catches
    /// bugs where verify() only checks global sum instead of per-asset.
    #[proptest]
    fn prop_multi_asset_cross_asset_shift_fails(
        #[strategy(multi_asset_refresh())] mut refresh: Refresh,
    ) {
        // Find first output for asset 0 and first output for asset 1
        let out_a = refresh.atoms.iter().enumerate().position(|(i, a)| {
            refresh.is_output(i) && a.asset_id == 0
        });
        let out_b = refresh.atoms.iter().enumerate().position(|(i, a)| {
            refresh.is_output(i) && a.asset_id == 1
        });

        if let (Some(ia), Some(ib)) = (out_a, out_b) {
            // Shift 1 unit from asset A output to asset B output
            refresh.atoms[ia].amount = refresh.atoms[ia].amount.saturating_sub(1);
            refresh.atoms[ib].amount = refresh.atoms[ib].amount.saturating_add(1);
            // Global sum unchanged, but per-asset balance broken
            prop_assert!(refresh.verify().is_err());
        }
    }
}
