use plonky2::{hash::hash_types::HashOutTarget, iop::target::Target};
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

use super::*;
use crate::{unwind_panic, Error};

/// Consumes a note, creating a new one with another Nonce
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Redeem {
    pub input: SealedNote,
    pub output: Note,
}

impl Arbitrary for Redeem {
    type Parameters = SealedNote;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(input: Self::Parameters) -> Self::Strategy {
        any::<Hash>()
            .prop_map(move |nonce| Self {
                input: input.clone(),
                output: Note {
                    asset_id: input.note.asset_id,
                    asset_name: input.note.asset_name,
                    amount: input.note.amount,
                    nonce,
                },
            })
            .boxed()
    }

    fn arbitrary() -> Self::Strategy {
        any::<SealedNote>()
            .prop_flat_map(Self::arbitrary_with)
            .boxed()
    }
}

impl EncodeFields for Redeem {
    fn as_fields(&self) -> Vec<F> {
        [self.input.note.as_fields(), self.output.as_fields()].concat()
    }
}

pub struct Circuit {
    pub data: CircuitData,
    pub commitment: HashOutTarget,
    pub input_asset_id: HashOutTarget,
    pub input_asset_name: HashOutTarget,
    pub input_amount: Target,
    pub input_nonce: HashOutTarget,
    pub output_asset_id: HashOutTarget,
    pub output_asset_name: HashOutTarget,
    pub output_amount: Target,
    pub output_nonce: HashOutTarget,
}

impl Sealable for Redeem {
    type Circuit = Circuit;

    fn circuit() -> Self::Circuit {
        let mut builder = circuit_builder();

        let input_amount = builder.add_virtual_target();
        let input_asset_id = builder.add_virtual_hash();
        let input_asset_name = builder.add_virtual_hash();
        let input_nonce = builder.add_virtual_hash();

        let output_amount = builder.add_virtual_target();
        let output_asset_id = builder.add_virtual_hash();
        let output_asset_name = builder.add_virtual_hash();
        let output_nonce = builder.add_virtual_hash();

        // Ensure amounts, asset_ids and asset_names are the same
        builder.connect(input_amount, output_amount);
        builder.connect_hashes(input_asset_id, output_asset_id);
        builder.connect_hashes(input_asset_name, output_asset_name);

        // Ensure input and output nonces are different
        let mut are_nonces_diff = builder._true();

        for i in 0..4 {
            let is_eq = builder.is_equal(output_nonce.elements[i], input_nonce.elements[i]);
            let is_diff = builder.not(is_eq);
            are_nonces_diff = builder.and(are_nonces_diff, is_diff);
        }

        builder.assert_bool(are_nonces_diff);

        let commitment = circuit_hash_to_curve(
            &mut builder,
            &[
                vec![input_amount],
                input_asset_id.elements.to_vec(),
                input_asset_name.elements.to_vec(),
                input_nonce.elements.to_vec(),
                vec![output_amount],
                output_asset_id.elements.to_vec(),
                output_asset_name.elements.to_vec(),
                output_nonce.elements.to_vec(),
            ]
            .concat(),
        );
        builder.register_public_inputs(&commitment.elements);

        Circuit {
            data: builder.build::<C>(),
            commitment,
            input_asset_id,
            input_asset_name,
            input_amount,
            input_nonce,
            output_asset_id,
            output_asset_name,
            output_amount,
            output_nonce,
        }
    }

    fn circuit_data() -> CircuitData {
        Self::circuit().data
    }

    fn prove(&self) -> Result<Proof, crate::Error> {
        let circuit = Self::circuit();

        let mut pw = PartialWitness::new();

        pw.set_target(
            circuit.input_amount,
            F::from_canonical_u64(self.input.note.amount),
        );
        pw.set_hash_target(circuit.input_asset_id, self.input.note.asset_id.into());
        pw.set_hash_target(circuit.input_asset_name, self.input.note.asset_name.into());
        pw.set_hash_target(circuit.input_nonce, self.input.note.nonce.into());
        pw.set_target(
            circuit.output_amount,
            F::from_canonical_u64(self.output.amount),
        );
        pw.set_hash_target(circuit.output_asset_id, self.output.asset_id.into());
        pw.set_hash_target(circuit.output_asset_name, self.output.asset_name.into());
        pw.set_hash_target(circuit.output_nonce, self.output.nonce.into());

        let commitment = PoseidonHash::hash_no_pad(
            &[self.input.note.as_fields(), self.output.as_fields()].concat(),
        );
        pw.set_hash_target(circuit.commitment, commitment);

        unwind_panic!(circuit.data.prove(pw).map_err(|e| Error::CryptoError {
            kind: e.root_cause().to_string(),
            reason: e.to_string(),
        }))
    }
}

impl ToMessage for Redeem {
    fn method() -> Method {
        Method::Redeem
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use crate::{protocol::*, Error};

    fn run(redeem: Redeem) -> Result<(), Error> {
        Redeem::verify(redeem.hash(), redeem.seal()?)
    }

    #[proptest(cases = 1)]
    fn test_prove(redeem: Redeem) {
        prop_assert!(run(redeem).is_ok())
    }

    fn mismatched_amount() -> impl Strategy<Value = Redeem> {
        any::<(Redeem, u64)>()
            .prop_map(|(mut r, a)| {
                r.output.amount = r.output.amount.wrapping_add(a);
                r
            })
            .prop_filter("redeem amounts must not match", |r: &Redeem| {
                r.input.note.amount != r.output.amount
            })
    }

    #[proptest(cases = 50)]
    fn test_prove_mismatched_amount(#[strategy(mismatched_amount())] redeem: Redeem) {
        prop_assert!(
            run(redeem).is_err(),
            "expected redeem with mismatched amounts to only generate invalid proofs."
        )
    }

    fn mismatched_asset_id() -> impl Strategy<Value = Redeem> {
        any::<(Redeem, Hash)>()
            .prop_map(|(mut r, asset_id)| {
                r.output.asset_id = asset_id;
                r
            })
            .prop_filter("redeem asset_ids must not match", |r: &Redeem| {
                r.input.note.asset_id != r.output.asset_id
            })
    }

    #[proptest(cases = 50)]
    fn test_prove_mismatched_asset_id(#[strategy(mismatched_asset_id())] redeem: Redeem) {
        prop_assert!(
            run(redeem).is_err(),
            "expected redeem with mismatched asset ids to only generate invalid proofs."
        )
    }

    fn mismatched_asset_name() -> impl Strategy<Value = Redeem> {
        any::<(Redeem, Name)>()
            .prop_map(|(mut r, asset_name)| {
                r.output.asset_name = asset_name;
                r
            })
            .prop_filter("redeem asset names must not match", |r: &Redeem| {
                r.input.note.asset_name != r.output.asset_name
            })
    }

    #[proptest(cases = 50)]
    fn test_prove_mismatched_asset_name(#[strategy(mismatched_asset_name())] redeem: Redeem) {
        prop_assert!(
            run(redeem).is_err(),
            "expected redeem with mismatched asset names to only generate invalid proofs."
        )
    }
}
