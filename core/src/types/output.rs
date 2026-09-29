use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use test_strategy::Arbitrary;

use crate::{
    crypto,
    error::Error,
    types::{
        Asset, AssetName, BlindSignature, DleqProofWithBlinding, Hash, Note,
        PolicyId, PublicKey, Signature,
    },
};

/// A blinded note that a client asks the delegate to sign. The asset and
/// amount are public, because the delegate signs with the key for that
/// asset and amount. The note nonce stays with the client.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Arbitrary,
)]
pub struct BlindedOutput {
    pub policy_id: PolicyId,
    pub asset_name: AssetName,
    pub amount: u64,
    /// The blinded point `B'`.
    pub point: Signature,
}

impl BlindedOutput {
    pub fn asset(&self) -> Asset {
        Asset {
            policy_id: self.policy_id,
            asset_name: self.asset_name,
        }
    }
}

/// The client side of a [`BlindedOutput`]: the note without its signature,
/// and the blinding factor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingNote {
    pub note: Note,
    pub blinding_factor: Hash,
}

/// Makes a new note of one denomination and blinds it.
pub fn blind_new_note<R: RngCore + CryptoRng>(
    rng: &mut R,
    delegate: PublicKey,
    asset: &Asset,
    amount: u64,
) -> (BlindedOutput, PendingNote) {
    let note = Note {
        amount,
        delegate,
        policy_id: asset.policy_id,
        asset_name: asset.asset_name,
        nonce: Hash::random(rng),
        signature: Signature::default(),
        dleq: None,
    };
    let blinded = crypto::blind_note(rng, &note);

    (
        BlindedOutput {
            policy_id: asset.policy_id,
            asset_name: asset.asset_name,
            amount,
            point: blinded.point.into(),
        },
        PendingNote {
            note,
            blinding_factor: blinded.factor.into(),
        },
    )
}

impl PendingNote {
    /// Unblinds the delegate's signature and checks its DLEQ proof with
    /// `public_key`, the delegate's key for the asset and amount.
    pub fn finish(
        self,
        signature: &BlindSignature,
        public_key: &PublicKey,
    ) -> Result<Note, Error> {
        let mut note = self.note;
        let proof = DleqProofWithBlinding {
            proof: signature.proof,
            blinding_factor: self.blinding_factor,
        };

        note.signature = crypto::unblind_signature(
            &signature.signature,
            &self.blinding_factor.to_scalar(),
            public_key,
        )?;

        if !crypto::verify_note_proof(
            public_key,
            note.commitment().as_ref(),
            note.signature,
            &proof,
        )? {
            return Err(Error::InvalidSignature {
                reason: "Invalid DLEQ proof for blinded output".to_string(),
                signature: note.signature,
            });
        }

        note.dleq = Some(proof);
        Ok(note)
    }
}

/// The bytes that a depositor signs (CIP-8) and hashes into the deposit
/// datum. They bind the deposit to its blinded outputs and their amounts.
pub fn deposit_intent_payload(
    outputs: &[BlindedOutput],
    delegate_pk: &PublicKey,
    script_address: &str,
    nonce: u64,
    network: &str,
) -> Vec<u8> {
    #[derive(Serialize)]
    struct Output {
        policy_id: String,
        asset_name: String,
        amount: u64,
        point: String,
    }

    #[derive(Serialize)]
    struct Payload<'a> {
        outputs: Vec<Output>,
        delegate_pk: String,
        script_address: &'a str,
        nonce: u64,
        network: &'a str,
    }

    let payload = Payload {
        outputs: outputs
            .iter()
            .map(|o| Output {
                policy_id: muhex::encode(o.policy_id.0),
                asset_name: muhex::encode(o.asset_name.as_bytes()),
                amount: o.amount,
                point: muhex::encode(o.point.0),
            })
            .collect(),
        delegate_pk: muhex::encode(delegate_pk.0),
        script_address,
        nonce,
        network,
    };

    serde_json::to_vec(&payload).expect("payload serializes")
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rand::{SeedableRng, rngs::StdRng};
    use test_strategy::proptest;

    use super::*;
    use crate::{keyset, types::SecretKey};

    #[proptest(cases = 64)]
    fn prop_pending_note_finishes_into_a_valid_note(
        master: SecretKey,
        asset: Asset,
        #[strategy(0u32..64)] d: u32,
        seed: [u8; 32],
    ) {
        let mut rng = StdRng::from_seed(seed);
        let amount = 1u64 << d;
        let (output, pending) =
            blind_new_note(&mut rng, master.public(), &asset, amount);

        prop_assert_eq!(output.amount, amount);
        prop_assert_eq!(output.asset(), asset);

        let signed = keyset::sign_blinded(
            &mut rng,
            &master,
            &asset,
            amount,
            &output.point.to_point()?,
        )?;
        let pair = keyset::denomination_keypair(&master, &asset, amount)?;
        let note = pending.finish(&signed, &pair.public_key)?;

        prop_assert!(keyset::verify(
            &master,
            &asset,
            amount,
            note.commitment().as_ref(),
            note.signature,
        )?);
        prop_assert!(note.dleq.is_some());
    }

    #[proptest(cases = 64)]
    fn prop_pending_note_rejects_the_wrong_key(
        master: SecretKey,
        asset: Asset,
        #[strategy(1u32..64)] d: u32,
        seed: [u8; 32],
    ) {
        let mut rng = StdRng::from_seed(seed);
        let amount = 1u64 << d;
        let (output, pending) =
            blind_new_note(&mut rng, master.public(), &asset, amount);

        // The delegate signs with the key for a smaller amount.
        let signed = keyset::sign_blinded(
            &mut rng,
            &master,
            &asset,
            amount >> 1,
            &output.point.to_point()?,
        )?;
        let pair = keyset::denomination_keypair(&master, &asset, amount)?;

        prop_assert!(pending.finish(&signed, &pair.public_key).is_err());
    }

    #[proptest]
    fn prop_deposit_intent_payload_binds_each_amount(
        output: BlindedOutput,
        delegate: PublicKey,
        nonce: u64,
        #[filter(#other != #output.amount)] other: u64,
    ) {
        let a = deposit_intent_payload(
            &[output],
            &delegate,
            "addr",
            nonce,
            "preprod",
        );
        let b = deposit_intent_payload(
            &[output],
            &delegate,
            "addr",
            nonce,
            "preprod",
        );
        let changed = BlindedOutput {
            amount: other,
            ..output
        };
        let c = deposit_intent_payload(
            &[changed],
            &delegate,
            "addr",
            nonce,
            "preprod",
        );

        prop_assert_eq!(&a, &b);
        prop_assert_ne!(a, c);
    }

    #[proptest]
    fn prop_deposit_intent_payload_binds_each_point(
        output: BlindedOutput,
        delegate: PublicKey,
        #[filter(#point != #output.point)] point: Signature,
    ) {
        let a =
            deposit_intent_payload(&[output], &delegate, "addr", 1, "preprod");
        let changed = BlindedOutput { point, ..output };
        let b =
            deposit_intent_payload(&[changed], &delegate, "addr", 1, "preprod");

        prop_assert_ne!(a, b);
    }
}
