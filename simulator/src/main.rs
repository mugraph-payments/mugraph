use color_eyre::eyre::Result;
use mugraph_simulator::{Config, Simulator};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let config = Config::default();
    let mut simulator = Simulator::new().setup(config).await?;

    loop {
        simulator.tick().await?;
    }

    #[allow(unreachable_code)]
    Ok(())
}
