use color_eyre::eyre::Result;
use mugraph_client::prelude::*;
use rand::{rngs::StdRng, Rng};

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

        for _ in 0..config.user_count {
            let mut user = User::new();

            for _ in 0..rng.gen_range(1..config.max_notes_per_user) {
                let idx = rng.gen_range(0..config.asset_count);

                let asset_id = assets[idx];
                let amount = rng.gen_range(1..1_000_000_000);

                let note = delegate.emit(&mut rng, asset_id, amount).await;

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
        todo!();
    }
}
