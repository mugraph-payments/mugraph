use color_eyre::eyre::Result;
use mugraph_node::{config::Config, start};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    start(&Config::new()).await?;

    Ok(())
}
