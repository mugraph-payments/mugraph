use axum::Router;
use color_eyre::eyre::Result;

mod context;
mod route;

pub async fn start() -> Result<()> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9999").await?;

    axum::serve(listener, Router::new().nest("v0", route::v0::router()?)).await?;

    Ok(())
}
