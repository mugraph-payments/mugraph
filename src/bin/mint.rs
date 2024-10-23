use mugraph::{mint::*, Error};

pub fn main() -> Result<(), Error> {
    let config = Config::load();

    Mint::new(&config)?.start::<Tcp>(config.listen_address)?;

    Ok(())
}
