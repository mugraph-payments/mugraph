use color_eyre::eyre::{ErrReport, Result};
use mugraph_client::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tokio::task;

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

    pub fn spawn(&mut self) {
        let seed = self.rng.gen();
        let keypair = self.keypair;

        task::spawn(async move {
            let config = mugraph_node::Config {
                seed: Some(seed),
                secret_key: Some(keypair.secret_key.to_string()),
                public_key: Some(keypair.public_key.to_string()),
                ..Default::default()
            };

            mugraph_node::start(&config).await?;

            Ok::<_, ErrReport>(())
        });
    }
}
