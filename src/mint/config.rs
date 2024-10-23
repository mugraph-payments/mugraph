use std::{
    env,
    fs::{self, File, OpenOptions},
    io::prelude::*,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use argh::FromArgs;

use super::{Decode, Encode, SecretKey, Tcp};
use crate::Error;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransportKind {
    #[default]
    Tcp,
    WebSockets,
}

#[derive(FromArgs, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Stuff
pub struct Config {
    #[argh(option, default = "default_db_path()")]
    /// the path where the database will be saved/loaded from
    pub database_path: PathBuf,

    #[argh(option)]
    /// the path to look for to find the secret key for this mint
    pub key_path: PathBuf,

    #[argh(switch)]
    /// whether or not to create the keypair if not found
    pub create_key: bool,

    #[argh(option, default = "default_transport()", from_str_fn(parse_transport))]
    /// which transport to use (TCP, UDP, WebSockets)
    pub transport: TransportKind,

    #[argh(option, default = "default_address()")]
    /// TCP Address to listen on for requests
    pub listen_address: SocketAddr,
}

impl Config {
    pub fn load() -> Self {
        argh::from_env()
    }

    pub fn transport(&self) -> impl super::Transport {
        match self.transport {
            TransportKind::Tcp => Tcp::new(self.listen_address),
            TransportKind::WebSockets => todo!(),
        }
    }

    pub fn secret_key(&self) -> Result<SecretKey, Error> {
        let bytes = match self.create_key {
            true if fs::exists(&self.key_path)? => {
                return Err(Error::MintConfiguration {
                    reason: "Asked for a new key to be created, but the output file already exists."
                        .to_string(),
                });
            }
            true => {
                let key = SecretKey::random();

                let mut file = File::create(&self.key_path)?;
                file.write(&key.as_bytes())?;

                key.as_bytes()
            }
            false if !fs::exists(&self.key_path)? => {
                return Err(Error::MintConfiguration {
                    reason: format!("Key file `{:?}` dies not exist", self.key_path),
                });
            }
            false => {
                let mut file = File::open(&self.key_path)?;

                let mut data = Vec::new();
                file.read_to_end(&mut data)?;

                data
            }
        };

        SecretKey::from_bytes(&bytes)
    }
}

fn default_db_path() -> PathBuf {
    let mut path = env::current_dir().expect("Failed to get current directory");
    path.push("db");
    path
}

fn default_transport() -> TransportKind {
    TransportKind::default()
}

fn default_address() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 4000)
}

fn parse_transport(input: &str) -> Result<TransportKind, String> {
    match input {
        "tcp" => Ok(TransportKind::Tcp),
        "ws" => Ok(TransportKind::WebSockets),
        t => Err(format!(
            "Invalid transport: {t}. Expected one of 'tcp', 'udp', 'ws'."
        )),
    }
}
