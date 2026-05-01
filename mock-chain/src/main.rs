use std::net::SocketAddr;

use clap::Parser;
use color_eyre::Result;
use mock_chain::{MineMode, Server, ServerConfig};

#[derive(Parser, Debug)]
#[command(
    about = "Blockfrost-compatible mock Cardano chain for the Mugraph demo"
)]
struct Args {
    /// Address to bind the HTTP server to.
    #[arg(long, default_value = "127.0.0.1:8090")]
    addr: SocketAddr,

    /// Mine a block automatically after each accepted submit. Disabling this
    /// requires explicit /admin/mine calls to confirm transactions.
    #[arg(long, default_value = "true")]
    auto_mine: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let mode = if args.auto_mine {
        MineMode::OnSubmit
    } else {
        MineMode::Manual
    };

    Server::new(ServerConfig {
        addr: args.addr,
        mode,
    })
    .run()
    .await
}
