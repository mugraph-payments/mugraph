use std::collections::HashMap;

use bonsai_bt::*;
use mugraph_client::prelude::*;

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

pub fn bt(id: u32, notes: Vec<Note>) -> BTUser {
    let send = Sequence(vec![Wait(1.0), Action(UserAction::Spend)]);

    let redeem = Sequence(vec![
        Action(UserAction::CheckRedeemable),
        Wait(1.0),
        Action(UserAction::Redeem),
    ]);

    let aggregate = Sequence(vec![
        Wait(1.0),
        Action(UserAction::CheckAggregate),
        Action(UserAction::Aggregate),
    ]);

    let mut user = User::new(id);

    for note in notes.into_iter() {
        user.note_by_asset_id
            .entry(note.asset_id)
            .or_insert_with(Vec::new)
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

pub fn tick(dt: f64, user: &mut BTUser) -> Option<Request> {
    let e: Event = UpdateArgs { dt }.into();

    let (status, dt) = user.tick(&e, &mut |args, user| match args.action {
        UserAction::Aggregate => (Status::Success, 0.0),
        UserAction::CheckAggregate => (Status::Success, 0.0),
        UserAction::CheckRedeemable => (Status::Success, 0.0),
        UserAction::Redeem => (Status::Success, 0.0),
        UserAction::Spend => (Status::Success, 0.0),
    });

    None
}
