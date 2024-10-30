use std::fs::{self, File};

use redb::Database;

use crate::{crypto::secret_to_public, protocol::*, Error};

mod config;
mod transport;

pub use self::{
    config::*,
    transport::{Tcp, Transport},
};

pub struct Mint {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
    pub config: config::Config,
    pub database: Database,
}

impl Mint {
    pub fn new(config: &config::Config) -> Result<Self, Error> {
        let secret_key = config.secret_key()?;
        let public_key = secret_to_public(secret_key)?;

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
            secret_key,
            public_key,
        })
    }

    pub fn start<T: Transport>(mut self, transport: T) -> Result<(), Error> {
        transport.listen(&mut self)?;
        Ok(())
    }

    pub fn apply(&mut self, message: &Message) -> Result<Vec<Signature>, Error> {
        match message.method {
            Method::Append => todo!(),
        }
    }
}
