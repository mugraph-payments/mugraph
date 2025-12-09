use color_eyre::eyre::Result;
use mugraph_node::{config::Config, start};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    let config = Config::new();
    let keypair = config.keypair()?;

    match &config {
        Config::GenerateKey => {
            info!(
                secret_key = %keypair.secret_key,
                public_key = %keypair.public_key,
                "No secret key supplied; generated one for this node. Pass --secret-key to reuse it."
            );
        }
        Config::Server {
            addr, secret_key, ..
        } => {
            if secret_key.is_none() {
                info!(
                    secret_key = %keypair.secret_key,
                    public_key = %keypair.public_key,
                    "No secret key supplied; generated one for this node. Pass --secret-key to reuse it."
                );
            }

            info!(addr = %addr, public_key = %keypair.public_key, "Starting server");

            start(*addr, keypair).await?;
        }
    }

    Ok(())
}
