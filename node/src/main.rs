use color_eyre::eyre::Result;
use mugraph_node::{config::Config, start};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();

    let config = Config::new();

    match &config {
        Config::GenerateKey => {
            // Print to stdout, not to the log: the operator must keep the
            // secret key and pass it to `server --secret-key`.
            let keypair = config.keypair()?;
            println!("secret_key={}", hex::encode(keypair.secret_key.0));
            println!("public_key={}", hex::encode(keypair.public_key.0));
        }
        Config::Server {
            addr, secret_key, ..
        } => {
            let keypair = config.keypair()?;
            if secret_key.is_none() {
                info!(
                    public_key = %keypair.public_key,
                    "No secret key supplied; generated one for this node. Pass --secret-key to reuse it."
                );
            }

            info!(addr = %addr, public_key = %keypair.public_key, "Starting server");

            start(*addr, config, keypair).await?;
        }
    }

    Ok(())
}
