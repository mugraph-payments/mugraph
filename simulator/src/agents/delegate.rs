use color_eyre::eyre::Result;
use crypto::generate_keypair;
use mugraph_client::prelude::*;
use rand::{CryptoRng, RngCore};

use crate::util::Location;

pub struct Delegate {
    pub location: Location,
    secret_key: SecretKey,
    public_key: PublicKey,
}

impl Delegate {
    pub fn new<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        let (secret_key, public_key) = generate_keypair(&mut rng);

        Self {
            location: Location::random(&mut rng),
            secret_key,
            public_key,
        }
    }

    pub async fn emit<R: RngCore + CryptoRng>(
        &self,
        mut rng: R,
        asset_id: Hash,
        amount: u64,
    ) -> Result<Note> {
        let mut note = Note {
            delegate: self.public_key,
            asset_id,
            nonce: Hash::random(&mut rng),
            amount,
            signature: Signature::default(),
        };

        note.signature =
            crypto::schnorr::sign(&mut rng, &self.secret_key, note.commitment().as_ref())?;

        Ok(note)
    }
}
