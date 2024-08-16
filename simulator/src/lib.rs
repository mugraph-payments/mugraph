use color_eyre::eyre::Result;
use mugraph_client::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

use self::agents::{delegate::Delegate, user};
pub use self::config::Config;

mod agents;
mod config;

#[allow(unused)]
pub struct Simulator {
    rng: ChaCha20Rng,
    delegate: Delegate,
    assets: Vec<Hash>,
    users: Vec<user::BTUser>,
}

impl Simulator {
    pub async fn build(config: Config) -> Result<Self> {
        let mut rng = config.rng();

        let delegate = Delegate::new(&mut rng);
        let assets = (0..config.assets)
            .map(|_| Hash::random(&mut rng))
            .collect::<Vec<_>>();
        let mut users = vec![];

        for i in 0..config.users {
            let mut notes = vec![];

            for _ in 0..rng.gen_range(1..config.notes) {
                let idx = rng.gen_range(0..config.assets);

                let asset_id = assets[idx];
                let amount = rng.gen_range(1..1_000_000_000);

                let note = delegate.emit(&mut rng, asset_id, amount).await?;

                notes.push(note);
            }

            users.push(user::bt(i as u32, notes));
        }

        Ok(Self {
            delegate: Delegate::new(&mut rng),
            rng,
            assets,
            users,
        })
    }

    pub async fn tick(&mut self) -> Result<()> {
        for user in self.users.iter_mut() {
            if let Some(req) = user::tick(1.0, user) {
                self.delegate.recv(req).await?;
            }
        }

        Ok(())
    }
}
