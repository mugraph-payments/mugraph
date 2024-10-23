use std::fs::{self, File};

use redb::Database;

use crate::{protocol::*, Error};

mod config;
mod transport;

pub use self::{
    config::*,
    transport::{Tcp, Transport},
};

pub struct Mint {
    pub config: config::Config,
    pub database: Database,
}

impl Mint {
    pub fn new(config: &config::Config) -> Result<Self, Error> {
        let database = match fs::exists(&config.database_path)? {
            true => redb::Builder::new().open(&config.database_path)?,
            false => {
                let file = File::create(&config.database_path)?;
                redb::Builder::new().create_file(file)?
            }
        };

        Ok(Self {
            config: config.clone(),
            database,
        })
    }

    pub fn start<T: Transport>(mut self, transport: T) -> Result<(), Error> {
        transport.listen(&mut self)?;
        Ok(())
    }

    pub fn apply(&mut self, message: &Message) -> Result<Vec<Signature>, Error> {
        match message.method {
            Method::Redeem => {
                todo!("Implement redeem logic");
            }
            Method::Append => todo!(),
        }
    }
}
