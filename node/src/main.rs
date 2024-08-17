use color_eyre::eyre::Result;
use mugraph_node::start;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    start().await?;

    Ok(())
}
