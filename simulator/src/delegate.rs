use color_eyre::eyre::Result;
use mugraph_core::{crypto, error::Error, types::*};
use mugraph_node::database::Database;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::info;

use crate::node::{Node, NodeTarget};

#[derive(Debug)]
pub struct Delegate {
    pub rng: ChaCha20Rng,
    pub db: Database,
    pub keypair: Keypair,
    pub node: Node,
}

impl Delegate {
    pub fn new<R: Rng + CryptoRng>(rng: &mut R, keypair: Keypair, target: Option<String>) -> Result<Self, Error> {
        let mut rng = ChaCha20Rng::seed_from_u64(rng.gen());

        info!(public_key = %keypair.public_key, "Starting delegate");
        let db = Database::setup_test(&mut rng, None)?;

        let node_target = match target {
          Some(endpoint) => NodeTarget::Remote(endpoint),
          None => NodeTarget::Local
        };

        let node = Node::new(node_target).unwrap();

        Ok(Self { db, rng, keypair, node })
    }

    #[tracing::instrument(skip_all)]
    pub fn emit(&mut self, asset_id: Hash, amount: u64) -> Result<Note, Error> {
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

    #[inline(always)]
    #[tracing::instrument(skip_all)]
    pub fn recv_transaction_v0(&mut self, tx: &Transaction) -> Result<V0Response, Error> {
      self.node.execute_transaction_v0(tx, self.keypair, &mut self.db)
    }
}
