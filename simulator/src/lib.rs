use color_eyre::eyre::Result;
use mugraph_client::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

use self::agents::*;
pub use self::config::Config;

mod agents;
mod config;
mod util;

pub struct Simulator {
    rng: StdRng,
    delegate: Delegate,
    assets: Vec<Hash>,
    users: Vec<User>,
}

impl Default for Simulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Simulator {
    pub fn new() -> Self {
        let mut rng = StdRng::from_entropy();

        Self {
            delegate: Delegate::new(&mut rng),
            rng,
            assets: vec![],
            users: vec![],
        }
    }

    pub async fn setup(mut self, config: Config) -> Result<Self> {
        self.rng = config.rng();

        self.delegate = Delegate::new(&mut self.rng);
        self.assets = (0..config.asset_count)
            .map(|_| Hash::random(&mut self.rng))
            .collect::<Vec<_>>();

        for _ in 0..config.user_count {
            let mut user = User::new(&mut self.rng);

            for _ in 0..self.rng.gen_range(1..config.max_notes_per_user) {
                let idx = self.rng.gen_range(0..config.asset_count);

                let asset_id = self.assets[idx];
                let amount = self.rng.gen_range(1..1_000_000_000);

                let note = self.delegate.emit(&mut self.rng, asset_id, amount).await?;

                user.notes.push(note);
            }

            self.users.push(user);
        }

        Ok(self)
    }

    pub async fn tick(&mut self) -> Result<()> {
        Ok(())
    }
}
