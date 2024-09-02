#![feature(duration_millis_float)]

use axum::Router;
use color_eyre::eyre::Result;

pub mod config;
pub mod database;
pub mod route;

pub use route::v0;

pub async fn start(config: &config::Config) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(config.addr).await?;

    axum::serve(
        listener,
        Router::new().nest("/v0", route::v0::router(config)?),
    )
    .await?;

    Ok(())
}
