use std::net::SocketAddr;

use color_eyre::eyre::Result;

pub mod cardano;
pub mod config;
pub mod database;
pub mod routes;

use mugraph_core::types::Keypair;

pub async fn start(addr: SocketAddr, keypair: Keypair) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, routes::router(keypair)?).await?;

    Ok(())
}
