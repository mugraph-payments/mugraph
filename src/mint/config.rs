use std::{env, net::SocketAddr, path::PathBuf};

use argh::FromArgs;

use super::Tcp;

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
    /// the path to look for to find the keypair for this mint
    pub keypair_path: PathBuf,

    #[argh(switch)]
    /// whether or not to create the keypair if not found
    pub create_keypair: bool,

    #[argh(option, from_str_fn(parse_transport))]
    /// which transport to use (TCP, UDP, WebSockets)
    pub transport: TransportKind,

    #[argh(option)]
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
}

fn default_db_path() -> PathBuf {
    let mut path = env::current_dir().expect("Failed to get current directory");
    path.push("db");
    path
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
