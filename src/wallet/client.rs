use std::{
    fmt,
    net::{self, SocketAddr},
    str,
};

use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    Resolver,
};

use crate::Error;

pub struct Client;

impl Client {
    pub fn connect(addr: [u8; 253], port: u16) -> Result<Connection, Error> {
        Connection::new(resolve(addr, port)?)
    }
}

pub struct Connection {
    pub addresses: Vec<SocketAddr>,
    pub stream: net::TcpStream,
}

impl Connection {
    pub fn new(addresses: Vec<SocketAddr>) -> Result<Self, Error> {
        let stream = net::TcpStream::connect(addresses.as_slice())
            .map_err(|e| Error::NetworkError(e.to_string()))?;

        Ok(Self { addresses, stream })
    }
}

pub fn resolve(addr: [u8; 253], port: u16) -> Result<Vec<SocketAddr>, Error> {
    let resolver = Resolver::new(ResolverConfig::cloudflare_tls(), ResolverOpts::default())?;
    let host = str::from_utf8(&addr)?.trim_end_matches('\0');

    Ok(resolver
        .lookup_ip(host)?
        .iter()
        .map(|x| (x, port).into())
        .collect())
}
