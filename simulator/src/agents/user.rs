use std::collections::HashMap;

use bonsai_bt::*;
use mugraph_client::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::*;

use crate::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserAction {
    Redeem,
    CheckSpend,
    Spend,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub friends: Vec<u32>,
    pub notes: Vec<Note>,
    pub note_by_asset_id: HashMap<Hash, Vec<u32>>,
    pub balance: HashMap<Hash, u64>,
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
            note_by_asset_id: Default::default(),
            balance: Default::default(),
            rng: ChaCha20Rng::from_entropy(),
        }
    }
}

pub type BTUser = BT<UserAction, User>;

pub fn bt(mut rng: ChaCha20Rng, id: u32, notes: Vec<Note>, config: &Config) -> BTUser {
    let send = If(
        Box::new(Action(UserAction::CheckSpend)),
        Box::new(Action(UserAction::Spend)),
        Box::new(Wait(3.0)),
    );

    let redeem = Sequence(vec![Wait(1.0), Action(UserAction::Redeem)]);

    let mut user = User::new(rng.clone(), id);

    user.friends = (1..config.users)
        .map(|_| rng.gen_range(0..config.users) as u32)
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

    BT::new(While(Box::new(WaitForever), vec![send, redeem]), user)
}

pub fn tick(dt: f64, mut user: BTUser) -> Result<BTUser> {
    let e: bonsai_bt::Event = UpdateArgs { dt }.into();

    user.tick(&e, &mut |args, bb| {
        let user = bb.get_db();

        match args.action {
            UserAction::Redeem => (Status::Success, 0.0),
            UserAction::CheckSpend => {
                if user.balance.is_empty() {
                    return (Status::Failure, 0.0);
                }

                (Status::Success, 1.0)
            }
            UserAction::Spend => {
                assert!(!user.friends.is_empty());

                let to = user.friends.choose(&mut user.rng).unwrap();
                let asset_id = user.balance.keys().choose(&mut user.rng).unwrap();
                let amount = user.rng.gen_range(1..user.balance[asset_id]);

                info!(from = user.id, to = to, asset_id = %asset_id, amount = amount , "Send");

                (Status::Success, 0.0)
            }
        }
    });

    Ok(user)
}
