use mugraph::{mint::*, Error};

pub fn main() -> Result<(), Error> {
    let config = Config::load();
    let mint = Mint::new(&config)?;

    server::tcp::start(mint, config.listen_address)?;

    Ok(())
}
