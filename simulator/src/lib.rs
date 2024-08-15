use color_eyre::eyre::Result;
use crypto::{blind, unblind_signature};
use mugraph_client::prelude::*;
use rand::{rngs::StdRng, Rng};
use tracing::{info, warn};

use self::agents::*;
pub use self::config::Config;

mod agents;
mod config;

#[allow(unused)]
pub struct Simulator {
    rng: StdRng,
    delegate: Delegate,
    assets: Vec<Hash>,
    users: Vec<User>,
}

impl Simulator {
    pub async fn build(config: Config) -> Result<Self> {
        let mut rng = config.rng();

        let delegate = Delegate::new(&mut rng);
        let assets = (0..config.asset_count)
            .map(|_| Hash::random(&mut rng))
            .collect::<Vec<_>>();
        let mut users = vec![];

        for i in 0..config.user_count {
            let mut user = User::new(i);

            for _ in 0..rng.gen_range(1..config.max_notes_per_user) {
                let idx = rng.gen_range(0..config.asset_count);

                let asset_id = assets[idx];
                let amount = rng.gen_range(1..1_000_000_000);

                let note = delegate.emit(&mut rng, asset_id, amount).await?;

                user.notes.push(note);
            }

            users.push(user);
        }

        Ok(Self {
            delegate: Delegate::new(&mut rng),
            rng,
            assets,
            users,
        })
    }

    pub async fn tick(&mut self) -> Result<()> {
        let total = self.users.len();
        let id = self.rng.gen_range(0..self.users.len());

        let nonce = Hash::random(&mut self.rng);
        let blinded = blind(&mut self.rng, nonce.as_ref());

        let (request, sender_id, note) = {
            let user = &mut self.users[id];

            if user.notes.is_empty() {
                warn!(sender_id = id, "User has no notes, skipping");
                return Ok(());
            }

            let mut note = user.notes.remove(self.rng.gen_range(0..user.notes.len()));

            note.nonce = nonce;

            let request = Request::Simple {
                inputs: vec![Input {
                    asset_id: note.asset_id,
                    amount: note.amount,
                    nonce: note.nonce,
                    signature: note.signature,
                }],
                outputs: vec![Output {
                    asset_id: note.asset_id,
                    amount: note.amount,
                    commitment: blinded.point.compress().into(),
                }],
            };

            (request, id, note)
        };

        let response = self.delegate.recv(request).await?;
        let receiver = &mut self.users[self.rng.gen_range(0..total)];

        info!(
            from = %sender_id,
            to = %receiver.id,
            asset_id = %note.asset_id,
            amount = note.amount,
            "Processed transaction"
        );

        for output in response.outputs {
            receiver.notes.push(Note {
                delegate: self.delegate.keypair.public_key,
                asset_id: note.asset_id,
                nonce,
                amount: note.amount,
                signature: unblind_signature(
                    &output,
                    &blinded.factor,
                    &self.delegate.keypair.public_key,
                )?,
            });
        }

        Ok(())
    }
}
