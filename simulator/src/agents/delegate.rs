use color_eyre::eyre::Result;
use crypto::{hash_to_curve, sign_blinded, verify};
use mugraph_client::prelude::*;
use rand_chacha::ChaCha20Rng;

pub struct Delegate {
    pub keypair: Keypair,
    pub rng: ChaCha20Rng,
}

impl Delegate {
    pub fn new(mut rng: ChaCha20Rng) -> Self {
        Self {
            keypair: Keypair::random(&mut rng),
            rng,
        }
    }

    pub async fn emit(&mut self, asset_id: Hash, amount: u64) -> Result<Note> {
        let mut note = Note {
            delegate: self.keypair.public_key,
            asset_id,
            nonce: Hash::random(&mut self.rng),
            amount,
            signature: Signature::default(),
        };

        let blind = crypto::blind_note(&mut self.rng, &note);
        let signed = crypto::sign_blinded(&self.keypair.secret_key, &blind.point);
        note.signature =
            crypto::unblind_signature(&signed, &blind.factor, &self.keypair.public_key)?;

        Ok(note)
    }

    pub async fn recv(&mut self, request: Request) -> Result<Response> {
        let mut signed_outputs = vec![];

        match request {
            Request::Simple { inputs, outputs } => {
                for input in inputs {
                    verify(
                        &self.keypair.public_key,
                        input.nonce.as_ref(),
                        input.signature,
                    )?;
                }

                for output in outputs {
                    let sig = sign_blinded(
                        &self.keypair.secret_key,
                        &hash_to_curve(output.commitment.0.as_ref()),
                    );

                    signed_outputs.push(sig);
                }
            }
        }

        Ok(Response {
            outputs: signed_outputs,
        })
    }
}
