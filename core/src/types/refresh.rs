use serde::{Deserialize, Serialize};

use rand::{CryptoRng, RngCore};

use super::{COMMITMENT_INPUT_SIZE, PublicKey};
use crate::{
    crypto,
    error::Error,
    keyset::is_denomination,
    types::{
        ASSET_ID_BYTES_SIZE, Asset, BlindSignature, Hash, Note, PendingNote,
        Signature, write_asset_bytes,
    },
    utils::BitSet32,
};

/// The input mask has 32 bits, so a refresh has 32 atoms or less.
pub const MAX_ATOMS: usize = 32;

/// The secret data that the client keeps for one blinded output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputSecret {
    /// The index of the output in `Refresh::atoms`.
    pub atom_index: usize,
    /// The note nonce. The client does not send it to the delegate.
    pub nonce: Hash,
    /// The blinding factor `r`.
    pub blinding_factor: Hash,
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
        write_asset_bytes(
            &asset.policy_id,
            &asset.asset_name,
            &mut asset_bytes,
        );
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
    #[serde(rename = "b", default, skip_serializing_if = "Vec::is_empty")]
    pub blinded_points: Vec<Signature>,
}

impl Refresh {
    pub fn is_input(&self, id: usize) -> bool {
        self.input_mask.contains(id as u32)
    }

    pub fn is_output(&self, id: usize) -> bool {
        !self.input_mask.contains(id as u32)
    }

    /// Blinds each output commitment and puts the points in
    /// `blinded_points`. Then it sets the nonce of each output atom to
    /// zero, so the delegate can not link the note when it is spent.
    pub fn blind_outputs<R: RngCore + CryptoRng>(
        &mut self,
        rng: &mut R,
    ) -> Vec<OutputSecret> {
        let mut secrets = Vec::new();
        let mut points = Vec::new();

        for i in 0..self.atoms.len() {
            if self.is_input(i) {
                continue;
            }

            let commitment = self.atoms[i].commitment(&self.asset_ids);
            let blinded = crypto::blind(rng, commitment.as_ref());

            secrets.push(OutputSecret {
                atom_index: i,
                nonce: self.atoms[i].nonce,
                blinding_factor: blinded.factor.into(),
            });
            points.push(Signature::from(blinded.point));
            self.atoms[i].nonce = Hash::zero();
        }

        self.blinded_points = points;
        secrets
    }

    /// Makes the note for one output from the delegate's blind signature.
    /// `public_key` is the delegate's key for the asset and amount of the
    /// output. Returns an error if the DLEQ proof is not valid.
    pub fn unblind_output(
        &self,
        secret: &OutputSecret,
        signature: &BlindSignature,
        public_key: &PublicKey,
    ) -> Result<Note, Error> {
        let atom = self.atoms.get(secret.atom_index).ok_or_else(|| {
            Error::InvalidAtom {
                reason: format!("No atom at index {}", secret.atom_index),
            }
        })?;
        let asset =
            self.asset_ids.get(atom.asset_id as usize).ok_or_else(|| {
                Error::InvalidAtom {
                    reason: format!("No asset at index {}", atom.asset_id),
                }
            })?;

        PendingNote {
            note: Note {
                amount: atom.amount,
                delegate: atom.delegate,
                policy_id: asset.policy_id,
                asset_name: asset.asset_name,
                nonce: secret.nonce,
                signature: Signature::default(),
                dleq: None,
            },
            blinding_factor: secret.blinding_factor,
        }
        .finish(signature, public_key)
    }

    pub fn verify(&self) -> Result<(), Error> {
        if self.atoms.len() > MAX_ATOMS {
            return Err(Error::InvalidOperation {
                reason: format!(
                    "Refresh has {} atoms, the maximum is {}",
                    self.atoms.len(),
                    MAX_ATOMS
                ),
            });
        }

        let mut pre = vec![0; self.asset_ids.len()];
        let mut post = vec![0; self.asset_ids.len()];

        for (i, atom) in self.atoms.iter().enumerate() {
            if !is_denomination(atom.amount) {
                return Err(Error::InvalidAtom {
                    reason: format!(
                        "Atom {} amount {} is not a power of two",
                        i, atom.amount
                    ),
                });
            }

            if self.is_input(i) {
                match atom.signature {
                    Some(s) if (s as usize) < self.signatures.len() => {}
                    _ => {
                        return Err(Error::InvalidAtom {
                            reason: format!(
                                "Atom {} is an input without a valid signature index",
                                i
                            ),
                        });
                    }
                }
            }

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
    use crate::{
        crypto,
        keyset::{self, split_amount},
        types::{Keypair, SecretKey},
    };

    /// Splits each amount into denominations, in order.
    fn denominations(amounts: &[u64]) -> Vec<u64> {
        amounts.iter().flat_map(|&a| split_amount(a)).collect()
    }

    /// Strategy that generates a structurally valid, balanced Refresh.
    fn balanced_refresh() -> impl Strategy<Value = Refresh> {
        (
            any::<PublicKey>(),
            any::<Asset>(),
            (0u32..=12).prop_map(|d| 1u64 << d),
            proptest::collection::vec(1u64..=500_000, 1..=2),
        )
            .prop_map(
                |(delegate, asset, input_amount, split_weights)| {
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
                    let output_amounts = denominations(&output_amounts);

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
                        blinded_points: vec![],
                    }
                },
            )
    }

    #[proptest]
    fn prop_balanced_refresh_verifies(
        #[strategy(balanced_refresh())] refresh: Refresh,
    ) {
        prop_assert!(refresh.verify().is_ok());
    }

    #[proptest]
    fn prop_unbalanced_refresh_fails(
        #[strategy(balanced_refresh())] mut refresh: Refresh,
    ) {
        // Find first output atom index
        let output_idx = refresh
            .atoms
            .iter()
            .enumerate()
            .position(|(i, _)| refresh.is_output(i));

        if let Some(idx) = output_idx {
            refresh.atoms[idx].amount =
                refresh.atoms[idx].amount.saturating_mul(2);
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
            (1u32..=6).prop_map(|d| 1u64 << d),
            (1u32..=6).prop_map(|d| 1u64 << d),
            proptest::collection::vec(1u64..=500_000, 1..=2),
            proptest::collection::vec(1u64..=500_000, 1..=2),
        )
            .prop_filter("assets must differ", |(_d, a, b, ..)| a != b)
            .prop_map(
                |(
                    delegate,
                    asset_a,
                    asset_b,
                    amount_a,
                    amount_b,
                    weights_a,
                    weights_b,
                )| {
                    fn split(amount: u64, weights: &[u64]) -> Vec<u64> {
                        let total_w: u64 = weights.iter().sum();
                        let mut out: Vec<u64> = weights
                            .iter()
                            .map(|w| amount * w / total_w)
                            .collect();
                        let sum: u64 = out.iter().sum();
                        if sum < amount {
                            out[0] += amount - sum;
                        }
                        out
                    }

                    let outputs_a = denominations(&split(amount_a, &weights_a));
                    let outputs_b = denominations(&split(amount_b, &weights_b));

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
                        signatures: vec![
                            Signature::default(),
                            Signature::default(),
                        ],
                        blinded_points: vec![],
                    }
                },
            )
    }

    /// Multi-asset balanced refresh must verify.
    ///
    /// Validates the per-asset balance check path with 2 independent assets.
    #[proptest]
    fn prop_multi_asset_balanced_verifies(
        #[strategy(multi_asset_refresh())] refresh: Refresh,
    ) {
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
        let out_a = refresh
            .atoms
            .iter()
            .enumerate()
            .position(|(i, a)| refresh.is_output(i) && a.asset_id == 0);
        let out_b = refresh
            .atoms
            .iter()
            .enumerate()
            .position(|(i, a)| refresh.is_output(i) && a.asset_id == 1);

        if let (Some(ia), Some(ib)) = (out_a, out_b) {
            // Move output A to asset B. The global sum does not change,
            // but the per-asset balance does.
            refresh.atoms[ia].asset_id = 1;
            let _ = ib;
            prop_assert!(refresh.verify().is_err());
        }
    }

    /// A balanced refresh must still fail when an amount is not a power
    /// of two, because no delegate key exists for that amount.
    #[proptest]
    fn prop_refresh_rejects_amounts_that_are_not_denominations(
        delegate: PublicKey,
        asset: Asset,
        #[filter(!#amount.is_power_of_two())] amount: u64,
    ) {
        let mut input_mask = BitSet32::new();
        input_mask.insert(0);
        let atom = |signature| Atom {
            delegate,
            asset_id: 0,
            amount,
            nonce: Hash::default(),
            signature,
        };

        let refresh = Refresh {
            input_mask,
            atoms: vec![atom(Some(0)), atom(None)],
            asset_ids: vec![asset],
            signatures: vec![Signature::default()],
            blinded_points: vec![],
        };

        prop_assert!(refresh.verify().is_err());
    }

    #[test]
    fn refresh_rejects_more_atoms_than_the_input_mask_holds() {
        let delegate = PublicKey::default();
        let mut atoms = vec![Atom {
            delegate,
            asset_id: 0,
            amount: 1 << 32,
            nonce: Hash::default(),
            signature: Some(0),
        }];
        atoms.extend((0..32).map(|_| Atom {
            delegate,
            asset_id: 0,
            amount: 1 << 27,
            nonce: Hash::default(),
            signature: None,
        }));
        let mut input_mask = BitSet32::new();
        input_mask.insert(0);

        let refresh = Refresh {
            input_mask,
            atoms,
            asset_ids: vec![Asset::default()],
            signatures: vec![Signature::default()],
            blinded_points: vec![],
        };

        assert!(refresh.verify().is_err());
    }

    #[proptest]
    fn prop_refresh_rejects_signature_index_out_of_range(
        #[strategy(balanced_refresh())] mut refresh: Refresh,
    ) {
        refresh.atoms[0].signature = Some(refresh.signatures.len() as u32);
        prop_assert!(refresh.verify().is_err());
    }

    #[proptest]
    fn prop_refresh_rejects_unsigned_input(
        #[strategy(balanced_refresh())] mut refresh: Refresh,
    ) {
        refresh.atoms[0].signature = None;
        prop_assert!(refresh.verify().is_err());
    }

    #[proptest]
    fn prop_blind_outputs_hides_output_nonces(
        #[strategy(balanced_refresh())] mut refresh: Refresh,
        seed: [u8; 32],
    ) {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::from_seed(seed);
        let before = refresh.clone();

        let secrets = refresh.blind_outputs(&mut rng);

        let outputs: Vec<usize> = (0..refresh.atoms.len())
            .filter(|&i| refresh.is_output(i))
            .collect();
        prop_assert_eq!(secrets.len(), outputs.len());
        prop_assert_eq!(refresh.blinded_points.len(), outputs.len());
        for (secret, &i) in secrets.iter().zip(&outputs) {
            prop_assert_eq!(secret.atom_index, i);
            prop_assert_eq!(secret.nonce, before.atoms[i].nonce);
            prop_assert_eq!(refresh.atoms[i].nonce, Hash::zero());
        }
    }

    /// The client blinds, the delegate signs with the denomination key,
    /// and the client gets a note that verifies with that key only.
    #[proptest(cases = 64)]
    fn prop_unblind_output_makes_a_valid_note(
        #[strategy(balanced_refresh())] mut refresh: Refresh,
        master: SecretKey,
        seed: [u8; 32],
    ) {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::from_seed(seed);
        let delegate = master.public();
        for atom in refresh.atoms.iter_mut() {
            atom.delegate = delegate;
            atom.nonce = Hash::random(&mut rng);
        }

        let secrets = refresh.blind_outputs(&mut rng);

        for (secret, point) in secrets.iter().zip(&refresh.blinded_points) {
            let atom = &refresh.atoms[secret.atom_index];
            let asset = &refresh.asset_ids[atom.asset_id as usize];
            let signed = keyset::sign_blinded(
                &mut rng,
                &master,
                asset,
                atom.amount,
                &point.to_point()?,
            )?;
            let pair =
                keyset::denomination_keypair(&master, asset, atom.amount)?;

            let note =
                refresh.unblind_output(secret, &signed, &pair.public_key)?;

            prop_assert_eq!(note.amount, atom.amount);
            prop_assert_eq!(note.nonce, secret.nonce);
            prop_assert!(keyset::verify(
                &master,
                asset,
                note.amount,
                note.commitment().as_ref(),
                note.signature,
            )?);
            prop_assert!(crypto::verify_note_proof(
                &pair.public_key,
                note.commitment().as_ref(),
                note.signature,
                note.dleq.as_ref().unwrap(),
            )?);
            prop_assert!(!crypto::verify(
                &master,
                note.commitment().as_ref(),
                note.signature,
            )?);
        }
    }

    #[proptest(cases = 64)]
    fn prop_unblind_output_rejects_a_signature_from_another_key(
        #[strategy(balanced_refresh())] mut refresh: Refresh,
        master: SecretKey,
        other: Keypair,
        seed: [u8; 32],
    ) {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::from_seed(seed);
        let secrets = refresh.blind_outputs(&mut rng);
        let secret = &secrets[0];
        let atom = &refresh.atoms[secret.atom_index];
        let asset = &refresh.asset_ids[atom.asset_id as usize];
        let point = refresh.blinded_points[0].to_point()?;

        let signed = crypto::sign_blinded(&mut rng, &other.secret_key, &point);
        let pair = keyset::denomination_keypair(&master, asset, atom.amount)?;

        prop_assert!(
            refresh
                .unblind_output(secret, &signed, &pair.public_key)
                .is_err()
        );
    }
}
