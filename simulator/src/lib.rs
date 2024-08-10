use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::{ErrReport, Result};
use mugraph_client::prelude::*;
use rand::{prelude::IteratorRandom, rngs::StdRng, Rng, SeedableRng};
use tokio::task::JoinSet;
use tracing::info;

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
    context: Context,
}

#[derive(Default)]
pub struct Context {
    pub user_distances: HashMap<(u32, u32), Duration>,
    pub user_delegate_distances: HashMap<u32, Duration>,
}

impl Simulator {
    pub fn new() -> Self {
        let mut rng = StdRng::from_entropy();

        Self {
            delegate: Delegate::new(&mut rng),
            rng,
            assets: vec![],
            users: vec![],
            context: Context::default(),
        }
    }

    pub async fn setup(mut self, config: Config) -> Result<Self> {
        self.rng = config.rng();

        self.delegate = Delegate::new(&mut self.rng);
        self.assets = (0..config.asset_count)
            .map(|_| Hash::random(&mut self.rng))
            .collect::<Vec<_>>();

        info!(count = self.assets.len(), "Initialized assets");

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

        info!(count = self.users.len(), "Initialized users");

        for i in 0..self.users.len() {
            for j in 0..self.users.len() {
                if i == j {
                    continue;
                }

                self.context.user_distances.insert(
                    (i as u32, j as u32),
                    self.users[i].location.latency_to(&self.users[j].location),
                );
            }

            self.context.user_delegate_distances.insert(
                i as u32,
                self.users[i].location.latency_to(&self.delegate.location),
            );
        }

        Ok(self)
    }

    pub async fn spawn(self) -> Result<()> {
        let mut set = JoinSet::new();

        for _ in self.users {
            set.spawn_local(async { Ok::<_, ErrReport>(()) });
        }

        Ok(())
    }

    pub async fn tick(&mut self) -> Result<()> {
        let i = self.rng.gen_range(0..self.users.len());
        let user = &mut self.users[i];

        user.tick(&self.context).await?;

        Ok(())
    }
}
