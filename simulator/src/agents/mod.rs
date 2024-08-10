use async_trait::async_trait;
use color_eyre::eyre::Result;

mod delegate;
mod user;

pub use self::{delegate::Delegate, user::User};
use crate::Context;

#[async_trait]
pub trait Agent {
    type Input;
    type Output;

    async fn recv(&mut self, message: Self::Input) -> Result<Self::Output>;
    async fn tick(&mut self, _context: &Context) -> Result<()> {
        Ok(())
    }
}
