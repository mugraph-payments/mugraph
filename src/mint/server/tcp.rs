use std::net::{SocketAddr, TcpListener, TcpStream};

use serde_cbor::from_reader;

use crate::{mint::Mint, protocol::*, Error};

pub fn start(mut mint: Mint, address: SocketAddr) -> Result<(), Error> {
    let listener = TcpListener::bind(address)?;

    println!("Server listening on {}", listener.local_addr()?);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle(&mut mint, stream) {
                    eprintln!("Error handling client: {}", e);
                }
            }
            Err(e) => eprintln!("Error accepting connection: {}", e),
        }
    }

    Ok(())
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
