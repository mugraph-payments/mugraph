use std::{collections::HashMap, sync::Arc};

use bonsai_bt::*;
use mugraph_client::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    RwLock,
};
use tracing::*;

use crate::{Config, Context};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserAction {
    CheckSpend,
    Spend,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub friends: Vec<Sender<Note>>,
    pub notes: Vec<Note>,
    pub note_by_asset_id: HashMap<Hash, Vec<u32>>,
    pub balance: HashMap<Hash, u64>,
    pub rng: ChaCha20Rng,
    pub rx: Arc<RwLock<Receiver<Note>>>,
}

impl User {
    pub fn new(rng: ChaCha20Rng, id: u32) -> Self {
        Self {
            id,
            rng,
            ..Self::default()
        }
    }
}

impl Default for User {
    fn default() -> Self {
        let (_, rx) = tokio::sync::mpsc::channel(10);
        Self {
            id: Default::default(),
            friends: Default::default(),
            notes: Default::default(),
            note_by_asset_id: Default::default(),
            balance: Default::default(),
            rng: ChaCha20Rng::from_entropy(),
            rx: Arc::new(rx.into()),
        }
    }
}

pub type BTUser = BT<UserAction, User>;

pub fn bt(
    mut rng: ChaCha20Rng,
    id: u32,
    context: &Context,
    notes: Vec<Note>,
    rx: Receiver<Note>,
    config: &Config,
) -> BTUser {
    let send = If(
        Box::new(Action(UserAction::CheckSpend)),
        Box::new(Action(UserAction::Spend)),
        Box::new(Wait(3.0)),
    );

    let mut user = User::new(rng.clone(), id);
    user.rx = Arc::new(rx.into());

    user.friends = (1..config.users)
        .map(|_| context.senders[rng.gen_range(0..config.users) as usize].clone())
        .collect();

    for note in notes.into_iter() {
        user.note_by_asset_id
            .entry(note.asset_id)
            .or_default()
            .push(user.notes.len() as u32);

        user.balance
            .entry(note.asset_id)
            .and_modify(|e| *e += note.amount)
            .or_insert(note.amount);

        user.notes.push(note);
    }

    BT::new(While(Box::new(WaitForever), vec![send]), user)
}

pub async fn tick(dt: f64, mut user: BTUser) -> Result<BTUser> {
    let e: bonsai_bt::Event = UpdateArgs { dt }.into();

    let u = user.get_blackboard().get_db();
    while let Ok(note) = u.rx.write().await.try_recv() {
        u.notes.push(note);
    }

    user.tick(&e, &mut |args, bb| {
        let user = bb.get_db();

        match args.action {
            UserAction::CheckSpend => {
                if user.balance.is_empty() {
                    return (Status::Failure, 0.0);
                }

                (Status::Success, 1.0)
            }
            UserAction::Spend => {
                assert!(!user.friends.is_empty());

                let to_idx = user.rng.gen_range(0..user.friends.len());
                let to = user.friends[to_idx].clone();

                let asset_id = user.balance.keys().choose(&mut user.rng).unwrap();
                let amount = user.rng.gen_range(1..user.balance[asset_id]);

                info!(from = %user.id, to = to_idx, asset_id = %asset_id, amount = amount , "Send");

                (Status::Success, 0.0)
            }
        }
    });

    Ok(user)
}
