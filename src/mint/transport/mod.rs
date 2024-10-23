use super::Mint;
use crate::Error;

mod tcp;

pub use tcp::Tcp;

pub trait Transport {
    type Params;

    fn start(&self, mint: &mut Mint) -> Result<(), Error>;
}
