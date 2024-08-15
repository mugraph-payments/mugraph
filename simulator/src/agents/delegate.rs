use color_eyre::eyre::Result;
use mugraph_client::prelude::*;
use rand::{CryptoRng, RngCore};

pub struct Delegate {
    keypair: Keypair,
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
    ) -> Note {
        let mut note = Note {
            delegate: self.keypair.public_key,
            asset_id,
            nonce: Hash::random(&mut rng),
            amount,
            signature: Signature::default(),
        };

        note.signature = crypto::schnorr::sign(
            &mut rng,
            &self.keypair.secret_key,
            note.commitment().as_ref(),
        );

        note
    }

    pub async fn recv(&mut self, _req: Request) -> Result<Response> {
        todo!();
    }
}
