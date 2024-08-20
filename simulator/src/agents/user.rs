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
    pub rx: Arc<RwLock<Receiver<Note>>>,
    pub rng: ChaCha20Rng,
    pub queue: Vec<(u32, Note)>,
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
        Self {
            id: Default::default(),
            friends: Default::default(),
            notes: Default::default(),
            rng: ChaCha20Rng::from_entropy(),
            rx: Arc::new(tokio::sync::mpsc::channel(10).1.into()),
            queue: Default::default(),
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
    user.notes = notes;

    BT::new(While(Box::new(WaitForever), vec![send]), user)
}

pub async fn tick(dt: f64, mut user: BTUser) -> Result<BTUser> {
    let e: bonsai_bt::Event = UpdateArgs { dt }.into();

    let mut u = user.get_blackboard().get_db();

    while let Ok(note) = u.rx.write().await.try_recv() {
        u.notes.push(note);
    }

    while let Some((idx, note)) = u.queue.pop() {
        u.friends[idx as usize].send(note).await.unwrap();
    }

    user.tick(&e, &mut |args, bb| {
        let user = bb.get_db();

        match args.action {
            UserAction::CheckSpend => {
                if user.notes.is_empty() {
                    return (Status::Failure, 0.0);
                }

                (Status::Success, 1.0)
            }
            UserAction::Spend => {
                assert!(!user.friends.is_empty());

                let to_idx = user.rng.gen_range(0..user.friends.len());
                let to = user.friends[to_idx].clone();

                let mut builder = TransactionBuilder::new(GreedyCoinSelection);

                // Select a random note to spend
                let note_idx = user.rng.gen_range(0..user.notes.len());
                let note = user.notes.remove(note_idx);

                // Decide on a random amount to spend (between 1 and the note's amount)
                let spend_amount = user.rng.gen_range(1..=note.amount);

                // Add the input note to the transaction
                builder = builder.input(note.clone());

                // Add the output for the recipient
                builder = builder.output(note.asset_id, spend_amount);

                // If there's change, add it as an output back to the sender
                if spend_amount < note.amount {
                    builder = builder.output(note.asset_id, note.amount - spend_amount);
                }

                info!("User {} spent {} to friend", user.id, spend_amount);

                (Status::Success, 0.0)
            }
        }
    });

    Ok(user)
}
