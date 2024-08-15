use color_eyre::eyre::Result;
use crypto::{hash_to_curve, sign_blinded, verify};
use mugraph_client::prelude::*;
use rand::{CryptoRng, RngCore};

pub struct Delegate {
    pub keypair: Keypair,
}

impl Delegate {
    pub fn new<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        Self {
            keypair: Keypair::random(&mut rng),
        }
    }

    pub async fn emit<R: RngCore + CryptoRng>(
        &self,
        mut rng: R,
        asset_id: Hash,
        amount: u64,
    ) -> Result<Note> {
        let mut note = Note {
            delegate: self.keypair.public_key,
            asset_id,
            nonce: Hash::random(&mut rng),
            amount,
            signature: Signature::default(),
        };

        let blind = crypto::blind_note(&mut rng, &note);
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
                    let verification = verify(
                        &self.keypair.public_key,
                        input.nonce.as_ref(),
                        input.signature,
                    )?;

                    if !verification {
                        return Ok(Response::Error {
                            message: "Invalid signature".to_string(),
                        });
                    }
                }

                for output in outputs {
                    let sig = sign_blinded(
                        &self.keypair.secret_key,
                        &hash_to_curve(output.commitment.as_ref()),
                    );

                    signed_outputs.push(sig);
                }
            }
        }

        Ok(Response::Success {
            outputs: signed_outputs,
        })
    }
}
