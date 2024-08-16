use std::collections::HashMap;

use bonsai_bt::*;
use mugraph_client::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use tracing::*;

use crate::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserAction {
    Aggregate,
    CheckAggregate,
    CheckRedeemable,
    Redeem,
    Spend,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct User {
    pub id: u32,
    pub friends: Vec<u32>,
    pub notes: Vec<Note>,
    pub note_by_asset_id: HashMap<Hash, Vec<u32>>,
    pub notes_unredeemed: Vec<u32>,
    pub balance_cleared: HashMap<Hash, u64>,
    pub balance_unredeemed: HashMap<Hash, u64>,
}

impl User {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            ..Self::default()
        }
    }
}

pub type BTUser = BT<UserAction, User>;

pub fn bt(rng: &mut ChaCha20Rng, id: u32, notes: Vec<Note>, config: &Config) -> BTUser {
    let send = Sequence(vec![Wait(1.0), Action(UserAction::Spend)]);

    let redeem = Sequence(vec![
        Wait(1.0),
        Action(UserAction::CheckRedeemable),
        Wait(1.0),
        Action(UserAction::Redeem),
    ]);

    let aggregate = Sequence(vec![
        Wait(1.0),
        Action(UserAction::CheckAggregate),
        Wait(1.0),
        Action(UserAction::Aggregate),
    ]);

    let mut user = User::new(id);

    user.friends = (1..config.users)
        .map(|_| rng.gen_range(0..config.users) as u32)
        .collect();

    for note in notes.into_iter() {
        user.note_by_asset_id
            .entry(note.asset_id)
            .or_default()
            .push(user.notes.len() as u32 - 1);
        user.balance_cleared
            .entry(note.asset_id)
            .and_modify(|e| *e += note.amount)
            .or_insert(note.amount);
        user.notes.push(note);
    }

    BT::new(
        While(Box::new(WaitForever), vec![send, redeem, aggregate]),
        user,
    )
}

pub fn tick(dt: f64, rng: &mut ChaCha20Rng, user: &mut BTUser) -> Result<()> {
    let e: bonsai_bt::Event = UpdateArgs { dt }.into();

    user.tick(&e, &mut |args, bb| {
        let user = bb.get_db();

        match args.action {
            UserAction::Aggregate => (Status::Success, 0.0),
            UserAction::CheckAggregate => (Status::Success, 0.0),
            UserAction::CheckRedeemable => (Status::Success, 0.0),
            UserAction::Redeem => (Status::Success, 0.0),
            UserAction::Spend => {
                assert!(!user.friends.is_empty());

                let to = user.friends.choose(rng).unwrap();
                let asset_id = user.balance_cleared.keys().choose(rng).unwrap();
                let amount = rng.gen_range(1..user.balance_cleared[asset_id]);

                info!(from = user.id, to = to, asset_id = %asset_id, amount = amount , "Send");

                (Status::Success, 0.0)
            }
        }
    });

    Ok(())
}
