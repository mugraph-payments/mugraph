use color_eyre::eyre::Result;
use mugraph_client::prelude::*;
use mugraph_node::{v0::transaction_v0, Context};
use rand_chacha::ChaCha20Rng;

#[derive(Debug, Clone)]
pub struct Delegate {
    pub keypair: Keypair,
    pub rng: ChaCha20Rng,
    pub context: Context,
}

impl Delegate {
    pub fn new(mut rng: ChaCha20Rng) -> Self {
        Self {
            keypair: Keypair::random(&mut rng),
            context: Context::new(&mut rng).unwrap(),
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

    pub fn recv_transaction_v0(&mut self, tx: Transaction) -> Result<V0Response> {
        Ok(transaction_v0(tx, &mut self.context)?)
    }
}
