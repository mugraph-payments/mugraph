use std::sync::mpsc::{channel, Receiver};

use bonsai_bt::*;
use itertools::Itertools;
use mugraph_core::{builder::*, crypto, error::Result, types::*};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::*;

use super::delegate::Delegate;
use crate::{Config, Context};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserAction {
    CheckSpend,
    Spend,
}

#[derive(Debug)]
pub struct User {
    pub id: u32,
    pub friends: Vec<u32>,
    pub notes: Vec<Note>,
    pub rx: Receiver<Note>,
    pub rng: ChaCha20Rng,
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
            rx: channel().1,
        }
    }
}

pub type BTUser = BT<UserAction, User>;

pub fn bt(
    mut rng: ChaCha20Rng,
    id: u32,
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
    user.rx = rx;

    user.friends = (1..config.users)
        .map(|_| rng.gen_range(0..config.users) as u32)
        .dedup()
        .collect_vec();
    user.notes = notes;

    BT::new(While(Box::new(WaitForever), vec![send]), user)
}

pub async fn tick(
    dt: f64,
    mut delegate: Delegate,
    context: Context,
    mut user: BTUser,
) -> Result<BTUser> {
    let e: bonsai_bt::Event = UpdateArgs { dt }.into();

    let u = user.get_blackboard().get_db();

    while let Ok(note) = u.rx.try_recv() {
        u.notes.push(note);
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

                // Select a random friend to send funds to
                let to_idx = user.rng.gen_range(0..user.friends.len());
                let to = user.friends[to_idx];

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

                // Build the transaction
                let transaction = builder.build().expect("Failed to build transaction");

                // Send the transaction to the delegate
                let response = delegate
                    .recv_transaction_v0(transaction)
                    .expect("Failed to send transaction");

                match response {
                    V0Response::Transaction { outputs } => {
                        // Create new notes from the outputs
                        for (i, blinded_sig) in outputs.iter().enumerate() {
                            let new_note = Note {
                                amount: if i == 0 {
                                    spend_amount
                                } else {
                                    note.amount - spend_amount
                                },
                                delegate: note.delegate,
                                asset_id: note.asset_id,
                                nonce: Hash::random(&mut user.rng),
                                signature: crypto::unblind_signature(
                                    blinded_sig,
                                    &crypto::blind(&mut user.rng, &[]).factor,
                                    &delegate.public_key(),
                                )
                                .expect("Failed to unblind signature"),
                            };

                            if i == 0 {
                                // Send the spent amount to the recipient
                                context
                                    .send_to(to as usize, new_note)
                                    .expect("Failed to send note");
                            } else {
                                // Keep the change
                                user.notes.push(new_note);
                            }
                        }
                    }
                }

                info!("User {} spent {} to {}", user.id, spend_amount, to);

                (Status::Success, 0.0)
            }
        }
    });

    Ok(user)
}
