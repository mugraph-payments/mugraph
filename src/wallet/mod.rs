#![allow(unused)]

use std::fs::{self, File};

use redb::Database;

use crate::{protocol::Note, Error};

pub mod client;

pub struct Wallet {
    db: Database,
}

impl Wallet {
    pub fn new(path: &str) -> Result<Self, Error> {
        let file = File::create(path)?;
        let db = redb::Builder::new().create_file(file)?;

        Ok(Self { db })
    }

    pub fn load(path: &str) -> Result<Self, Error> {
        let db = redb::Builder::new().open(path)?;

        Ok(Self { db })
    }

    pub fn load_or_create(path: &str) -> Result<Self, Error> {
        if fs::exists(path)? {
            Self::load(path)
        } else {
            Self::new(path)
        }
    }
}
