use super::Mint;
use crate::Error;

mod tcp;

pub use tcp::Tcp;

pub trait Transport {
    type Params;

    fn start(mint: &mut Mint, params: Self::Params) -> Result<(), Error>;
}
