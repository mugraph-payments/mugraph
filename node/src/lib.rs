use std::net::SocketAddr;

use color_eyre::eyre::Result;

pub mod cardano;
pub mod config;
pub mod database;
pub(crate) mod deposit_datum;
pub mod deposit_monitor;
pub mod lifecycle;
pub(crate) mod network;
pub mod observability;
pub mod peer_registry;
pub mod provider;
pub mod reconciler;
pub mod routes;
pub(crate) mod tx_ids;
pub mod tx_signer;

use config::Config;

pub async fn start(addr: SocketAddr, config: Config) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, routes::router(config).await?).await?;

    Ok(())
}
