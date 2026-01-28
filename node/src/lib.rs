use std::net::SocketAddr;

use color_eyre::eyre::Result;

pub mod cardano;
pub mod config;
pub mod database;
pub mod deposit_monitor;
pub mod provider;
pub mod routes;
pub mod tx_signer;

use mugraph_core::types::Keypair;

pub async fn start(addr: SocketAddr, keypair: Keypair) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, routes::router(keypair).await?).await?;

    Ok(())
}
