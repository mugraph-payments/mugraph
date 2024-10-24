use std::fmt;

use plonky2::{
    hash::{hash_types::HashOutTarget, poseidon::PoseidonHash},
    iop::{
        target::{BoolTarget, Target},
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{circuit_data::CircuitConfig, config::Hasher},
};
use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use super::{DecodeFields, Hash, Name, PublicKey};
use crate::{protocol::*, unwind_panic, Decode, Encode, EncodeFields, Error};

#[derive(
    Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Arbitrary, Serialize, Deserialize,
)]
pub struct SealedNote {
    pub issuing_key: PublicKey,
    pub host: String,
    pub port: u16,
    pub note: Note,
    pub signature: Signature,
}

impl SealedNote {
    pub fn host(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Arbitrary, Serialize, Deserialize)]
pub struct Note {
    #[filter(#asset_id != Hash::zero())]
    pub asset_id: Hash,
    #[filter(#asset_name != Name::zero())]
    pub asset_name: Name,
    #[strategy(1u64..)]
    pub amount: u64,
    #[filter(#nonce != Hash::zero())]
    pub nonce: Hash,
}

impl Encode for Note {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.amount.to_le_bytes());
        bytes.extend_from_slice(&self.asset_id.as_bytes());
        bytes.extend_from_slice(&self.asset_name.as_bytes());
        bytes.extend_from_slice(&self.nonce.as_bytes());
        bytes
    }
}

impl EncodeFields for Note {
    fn as_fields(&self) -> Vec<F> {
        let mut fields = Vec::new();
        fields.push(F::from_canonical_u64(self.amount));
        fields.extend(self.asset_id.as_fields());
        fields.extend(self.asset_name.as_fields());
        fields.extend(self.nonce.as_fields());
        fields
    }
}

impl Decode for Note {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 8 + 32 * 2 + 32 {
            return Err(Error::DecodeError("Invalid size".to_string()));
        }

        let amount = u64::from_le_bytes(bytes[0usize..8].try_into().unwrap());
        let asset_id = Hash::from_bytes(&bytes[8..8 + 32])?;
        let asset_name = Name::from_bytes(&bytes[8 + 32..8 + 32 + 32])?;
        let nonce = Hash::from_bytes(&bytes[8 + 32 + 32..])?;

        Ok(Note {
            amount,
            asset_id,
            asset_name,
            nonce,
        })
    }
}

impl DecodeFields for Note {
    fn from_fields(fields: &[F]) -> Result<Self, Error> {
        if fields.len() < 1 + 4 + 4 + 4 {
            return Err(Error::DecodeError("Not enough fields for Note".to_string()));
        }

        let amount = fields[0].to_canonical_u64();
        let asset_id = Hash::from_fields(&fields[1..5])?;
        let asset_name = Name::from_fields(&fields[5..9])?;
        let nonce = Hash::from_fields(&fields[9..13])?;

        Ok(Note {
            amount,
            asset_id,
            asset_name,
            nonce,
        })
    }
}

impl Note {
    pub const BYTE_SIZE: usize = 32 * 3 + 8; // 3 Hash (32 bytes each) + 1 u64
    pub const FIELD_SIZE: usize = 4 * 3 + 1; // 3 Hash (4 fields each) + 1 field for amount

    pub fn asset_name(&self) -> String {
        self.asset_name.to_string()
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Debug for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Note")
            .field("asset_id", &self.asset_id)
            .field("asset_name", &self.asset_name())
            .field("amount", &self.amount)
            .field("nonce", &self.nonce)
            .finish()
    }
}

pub struct Circuit {
    data: CircuitData,
    asset_id: HashOutTarget,
    asset_name: HashOutTarget,
    amount: Target,
    nonce: HashOutTarget,
    commitment: HashOutTarget,
}

fn hash_is_zero(builder: &mut CircuitBuilder, hash: HashOutTarget) -> BoolTarget {
    let zero = builder.zero();
    let mut target = builder._true();

    for i in 0..hash.elements.len() {
        let is_zero = builder.is_equal(hash.elements[i], zero);
        target = builder.and(target, is_zero);
    }

    target
}

impl Sealable for Note {
    type Circuit = Circuit;

    fn circuit() -> Self::Circuit {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::new(config);

        let zero = builder.zero();

        // Private inputs
        let amount = builder.add_virtual_target();
        let asset_id = builder.add_virtual_hash();
        let asset_name = builder.add_virtual_hash();
        let nonce = builder.add_virtual_hash();

        // Public input
        let commitment = circuit_hash_to_curve(
            &mut builder,
            &[
                vec![amount],
                asset_id.elements.to_vec(),
                asset_name.elements.to_vec(),
                nonce.elements.to_vec(),
            ]
            .concat(),
        );
        builder.register_public_inputs(&commitment.elements);

        // Assert amount is non-zero
        let is_zero = builder.is_equal(amount, zero);
        builder.assert_zero(is_zero.target);

        let asset_id_not_zero = hash_is_zero(&mut builder, asset_id);
        builder.assert_zero(asset_id_not_zero.target);
        let asset_name_not_zero = hash_is_zero(&mut builder, asset_name);
        builder.assert_zero(asset_name_not_zero.target);
        let nonce_not_zero = hash_is_zero(&mut builder, nonce);
        builder.assert_zero(nonce_not_zero.target);

        let data = builder.build::<C>();

        Circuit {
            data,
            asset_id,
            asset_name,
            amount,
            nonce,
            commitment,
        }
    }

    fn circuit_data() -> CircuitData {
        Self::circuit().data
    }

    fn prove(&self) -> Result<Proof, Error> {
        let circuit = Self::circuit();

        let mut pw = PartialWitness::new();

        pw.set_target(circuit.amount, F::from_canonical_u64(self.amount));
        pw.set_hash_target(circuit.asset_id, self.asset_id.into());
        pw.set_hash_target(circuit.asset_name, self.asset_name.into());
        pw.set_hash_target(circuit.nonce, self.nonce.into());

        let commitment = PoseidonHash::hash_no_pad(&self.as_fields_with_prefix());
        pw.set_hash_target(circuit.commitment, commitment);

        unwind_panic!(circuit.data.prove(pw).map_err(|e| Error::CryptoError {
            kind: e.root_cause().to_string(),
            reason: e.to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::*;

    crate::test_encode_bytes!(Note);
    crate::test_encode_fields!(Note);

    fn run(note: Note) -> Result<(), Error> {
        Note::verify(note.hash(), note.seal()?)
    }

    #[proptest(cases = 100)]
    fn test_prove(note: Note) {
        prop_assert!(run(note).is_ok())
    }

    fn zero_amount() -> impl Strategy<Value = Note> {
        any::<Note>().prop_map(|mut n| {
            n.amount = 0;
            n
        })
    }

    #[proptest(cases = 10)]
    fn test_prove_zero_amount(#[strategy(zero_amount())] note: Note) {
        prop_assert!(
            run(note).is_err(),
            "Expected note with zero amount to only generate invalid proofs."
        )
    }

    fn zero_asset_id() -> impl Strategy<Value = Note> {
        any::<Note>().prop_map(|mut n| {
            n.asset_id = Hash::zero();
            n
        })
    }

    #[proptest(cases = 10)]
    fn test_prove_zero_asset_id(#[strategy(zero_asset_id())] note: Note) {
        prop_assert!(
            run(note).is_err(),
            "Expected note with empty asset_id to only generate invalid proofs."
        )
    }

    fn zero_asset_name() -> impl Strategy<Value = Note> {
        any::<Note>().prop_map(|mut n| {
            n.asset_name = Name::zero();
            n
        })
    }

    #[proptest(cases = 10)]
    fn test_prove_zero_asset_name(#[strategy(zero_asset_name())] note: Note) {
        prop_assert!(
            run(note).is_err(),
            "Expected note with empty asset_name to only generate invalid proofs."
        )
    }

    fn partial_zero_asset_id() -> impl Strategy<Value = Note> {
        (any::<Note>(), 0usize..4).prop_map(|(mut note, index): (Note, usize)| {
            let mut asset_id = note.asset_id.as_fields();
            asset_id[index] = F::ZERO;
            note.asset_id = Hash::from_fields(&asset_id).unwrap();
            note
        })
    }

    #[proptest(cases = 10)]
    // There was a bug where if you changed a hash so that one of the fields was zero, the proof
    // fails. This test ensures the behavior does not happen anymore.
    fn test_prove_asset_id_with_zero_bytes_slice(#[strategy(partial_zero_asset_id())] note: Note) {
        prop_assume!(note.asset_id != Hash::zero());

        prop_assert!(
            run(note).is_ok(),
            "Expected note to be valid even if a byte slice is empty, even though the whole hash isn't"
        )
    }

    fn partial_zero_asset_name() -> impl Strategy<Value = Note> {
        (any::<Note>(), 0usize..4).prop_map(|(mut note, index): (Note, usize)| {
            let mut asset_name = note.asset_name.as_fields();
            asset_name[index] = F::ZERO;
            note.asset_name = Name::from_fields(&asset_name).unwrap();
            note
        })
    }

    #[proptest(cases = 10)]
    // There was a bug where if you changed a hash so that one of the fields was zero, the proof
    // fails. This test ensures the behavior does not happen anymore.
    fn test_prove_asset_name_with_zero_bytes_slice(
        #[strategy(partial_zero_asset_name())] note: Note,
    ) {
        prop_assume!(note.asset_name != Name::zero());

        prop_assert!(
            run(note).is_ok(),
            "Expected note to be valid even if a byte slice is empty, even though the whole hash isn't"
        )
    }

    fn partial_zero_nonce() -> impl Strategy<Value = Note> {
        (any::<Note>(), 0usize..4).prop_map(|(mut note, index): (Note, usize)| {
            let mut nonce = note.nonce.as_fields();
            nonce[index] = F::ZERO;
            note.nonce = Hash::from_fields(&nonce).unwrap();
            note
        })
    }

    #[proptest(cases = 10)]
    // There was a bug where if you changed a hash so that one of the fields was zero, the proof
    // fails. This test ensures the behavior does not happen anymore.
    fn test_prove_nonce_with_zero_bytes_slice(#[strategy(partial_zero_nonce())] note: Note) {
        prop_assume!(note.nonce != Hash::zero());

        prop_assert!(
            run(note).is_ok(),
            "Expected note to be valid even if a byte slice is empty, even though the whole hash isn't"
        )
    }
}
