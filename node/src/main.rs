use color_eyre::eyre::Result;
use mugraph_node::{config::Config, start};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    match Config::new() {
        c @ Config::Server { addr, .. } => {
            let keypair = c.keypair()?;
            info!(addr = %addr, public_key = %keypair.public_key, "Starting server");

            start(addr, keypair).await?;
        }
    }

    Ok(())
}
