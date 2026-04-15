use proptest::prelude::*;
use rand::{prelude::*, rngs::StdRng};

use crate::{
    crypto,
    types::{
        AssetName, DleqProofWithBlinding, Hash, Keypair, Note, PolicyId,
        Signature,
    },
};

pub fn rng() -> impl Strategy<Value = StdRng> {
    any::<[u8; 32]>().prop_map(StdRng::from_seed)
}

/// Strategy that produces a cryptographically valid Note signed by the given keypair.
pub fn valid_note(keypair: Keypair) -> impl Strategy<Value = Note> {
    (
        rng(),
        any::<PolicyId>(),
        any::<AssetName>(),
        1u64..=1_000_000,
    )
        .prop_map(move |(mut rng, policy_id, asset_name, amount)| {
            let mut note = Note {
                delegate: keypair.public_key,
                policy_id,
                asset_name,
                nonce: Hash::random(&mut rng),
                amount,
                signature: Signature::default(),
                dleq: None,
            };

            let blind = crypto::blind_note(&mut rng, &note);
            let signed = crypto::sign_blinded(
                &mut rng,
                &keypair.secret_key,
                &blind.point,
            );
            note.signature = crypto::unblind_signature(
                &signed.signature,
                &blind.factor,
                &keypair.public_key,
            )
            .expect("unblind should succeed with valid inputs");
            note.dleq = Some(DleqProofWithBlinding {
                proof: signed.proof,
                blinding_factor: blind.factor.into(),
            });

            note
        })
}
