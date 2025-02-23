use color_eyre::eyre::Result;
use mugraph_node::{config::Config, start};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    match Config::new() {
        c @ Config::Server { addr, .. } => {
            start(addr, c.keypair()?).await?;
        }
    }

    Ok(())
}
