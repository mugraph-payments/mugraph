use crate::{proof::Proof, Error};

pub trait Event {
    type Input;

    fn recv(&self, input: Self::Input) -> Result<Proof, Error>;
}
