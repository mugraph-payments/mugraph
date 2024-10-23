use super::Mint;
use crate::Error;

mod tcp;

pub use tcp::Tcp;

pub trait Transport {
    type Params;

    fn listen(&self, mint: &mut Mint) -> Result<(), Error>;
}
