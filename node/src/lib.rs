use std::net::SocketAddr;

use axum::Router;
use color_eyre::eyre::Result;

pub mod config;
pub mod database;
pub mod route;

use mugraph_core::types::Keypair;
pub use route::v0;

pub async fn start(addr: SocketAddr, keypair: Keypair) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(
        listener,
        Router::new().nest("/v0", route::v0::router(keypair)?),
    )
    .await?;

    Ok(())
}
