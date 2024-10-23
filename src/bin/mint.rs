use mugraph::{mint::*, Error};

pub fn main() -> Result<(), Error> {
    let config = Config::load();

    Mint::new(&config)?.start(config.transport())?;

    Ok(())
}
