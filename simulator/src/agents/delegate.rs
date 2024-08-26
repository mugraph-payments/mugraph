use color_eyre::eyre::Result;
use mugraph_client::prelude::*;
use mugraph_node::{v0::transaction_v0, Context};
use rand_chacha::ChaCha20Rng;

#[derive(Debug, Clone)]
pub struct Delegate {
    pub context: Context,
}

impl Delegate {
    pub fn new(mut rng: ChaCha20Rng) -> Self {
        Self {
            context: Context::new(&mut rng).unwrap(),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.context.keypair.public_key
    }

    pub fn secret_key(&self) -> SecretKey {
        self.context.keypair.secret_key
    }

    pub async fn emit(&mut self, asset_id: Hash, amount: u64) -> Result<Note> {
        let mut note = Note {
            delegate: self.public_key(),
            asset_id,
            nonce: Hash::random(&mut self.context.rng),
            amount,
            signature: Signature::default(),
        };

        let blind = crypto::blind_note(&mut self.context.rng, &note);
        let signed = crypto::sign_blinded(&self.secret_key(), &blind.point);
        note.signature = crypto::unblind_signature(&signed, &blind.factor, &self.public_key())?;

        Ok(note)
    }

    pub fn recv_transaction_v0(&mut self, tx: Transaction) -> Result<V0Response> {
        Ok(transaction_v0(tx, &mut self.context)?)
    }
}
