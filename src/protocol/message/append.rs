use std::collections::HashMap;

use proptest::prelude::*;

use crate::{protocol::*, testing::distribute};

#[derive(Debug)]
pub struct Append<const I: usize, const O: usize> {
    pub inputs: [SealedNote; I],
    pub outputs: [Note; O],
}

#[derive(Debug, Clone)]
pub struct Payload {
    outputs: Vec<BlindedValue>,
}

impl<const I: usize, const O: usize> Append<I, O> {
    #[inline]
    pub fn payload(&self) -> Payload {
        todo!()
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        let mut pre = HashMap::new();
        let mut post = HashMap::new();

        for input in self.inputs.iter() {
            *pre.entry((input.note.asset_id, input.note.asset_name))
                .or_insert(0u128) += input.note.amount as u128;
        }

        for output in self.outputs.iter() {
            *post
                .entry((output.asset_id, output.asset_name))
                .or_insert(0u128) += output.amount as u128;
        }

        pre.values().sum::<u128>() == post.values().sum::<u128>()
    }
}

impl<const I: usize, const O: usize> Arbitrary for Append<I, O> {
    type Parameters = SecretKey;
    type Strategy = BoxedStrategy<Self>;

    #[inline]
    fn arbitrary_with(_secret_key: Self::Parameters) -> Self::Strategy {
        // Use `distribute` to generate balanced inputs and outputs
        distribute(I, O)
            .prop_map(move |(inputs, outputs)| {
                let inputs = inputs.into_iter().map(|_note| todo!()).collect::<Vec<_>>();

                Append {
                    inputs: inputs.try_into().unwrap(),
                    outputs: outputs.try_into().unwrap(),
                }
            })
            .boxed()
    }

    #[inline]
    fn arbitrary() -> Self::Strategy {
        any::<SecretKey>()
            .prop_flat_map(Self::arbitrary_with)
            .boxed()
    }
}
