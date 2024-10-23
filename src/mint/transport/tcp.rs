use std::net::{SocketAddr, TcpListener, TcpStream};

use serde_cbor::from_reader;

use super::Transport;
use crate::{mint::Mint, protocol::*, Error};

pub struct Tcp {
    listen_address: SocketAddr,
}

impl Tcp {
    pub fn new(listen_address: SocketAddr) -> Self {
        Self { listen_address }
    }
}

impl Transport for Tcp {
    type Params = SocketAddr;

    fn listen(&self, mint: &mut Mint) -> Result<(), Error> {
        let listener = TcpListener::bind(self.listen_address)?;

        println!("Server listening on {}", listener.local_addr()?);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = handle(mint, stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                }
                Err(e) => eprintln!("Error accepting connection: {}", e),
            }
        }

        Ok(())
    }
}

fn handle(mint: &mut Mint, mut stream: TcpStream) -> Result<(), Error> {
    loop {
        let message: Message = match from_reader(&mut stream) {
            Ok(event) => event,
            Err(e) if e.is_eof() => break,
            Err(e) => return Err(Error::DecodeError(e.to_string())),
        };

        mint.apply(&message)?;

        println!("Received event: {:?}", message);
    }

    Ok(())
}
