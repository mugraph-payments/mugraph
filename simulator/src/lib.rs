use std::collections::VecDeque;

use agents::user::BTUser;
use color_eyre::eyre::{ErrReport, Result};
use futures_util::future::try_join_all;
use mugraph_client::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tokio::sync::mpsc::{self, Sender};

use self::agents::{delegate::Delegate, user};
pub use self::config::Config;

mod agents;
mod config;

#[allow(unused)]
pub struct Simulator {
    rng: ChaCha20Rng,
    delegate: Delegate,
    timescale: f64,
    assets: Vec<Hash>,
    users: Vec<BTUser>,
    context: Context,
}

pub struct Context {
    pub senders: Vec<Sender<Note>>,
}

impl Simulator {
    pub async fn build(mut rng: ChaCha20Rng, config: Config) -> Result<Self> {
        let mut delegate = Delegate::new(rng.clone());
        let assets = (0..config.assets)
            .map(|_| Hash::random(&mut rng))
            .collect::<Vec<_>>();
        let mut users = vec![];
        let mut context = Context {
            senders: Vec::with_capacity(config.users),
        };
        let mut receivers = VecDeque::with_capacity(config.users);

        for _ in 0..config.users {
            let (tx, rx) = mpsc::channel(10);
            context.senders.push(tx);
            receivers.push_back(rx);
        }

        for i in 0..config.users {
            let mut notes = vec![];
            let rx = receivers.pop_front().unwrap();

            for _ in 0..rng.gen_range(1..config.notes) {
                let idx = rng.gen_range(0..config.assets);

                let asset_id = assets[idx];
                let amount = rng.gen_range(1..1_000_000_000);

                let note = delegate.emit(asset_id, amount).await?;

                notes.push(note);
            }

            assert_ne!(notes.len(), 0);

            users.push(user::bt(
                rng.clone(),
                i as u32,
                &context,
                notes,
                rx,
                &config,
            ));
        }

        delegate.spawn();

        Ok(Self {
            delegate,
            rng,
            assets,
            users,
            timescale: 1.0,
            context,
        })
    }

    pub async fn tick(mut self) -> Result<Self> {
        let timescale = self.timescale;

        self.users = try_join_all(self.users.into_iter().map(|u| async {
            let user = user::tick(timescale, u).await?;
            Ok(user)
        }))
        .await
        // This whole error handling thing was too much for poor old rust type system
        .map_err(|e: ErrReport| e)?;

        Ok(self)
    }
}
