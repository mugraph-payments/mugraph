mod message;
mod note;

pub use self::{
    message::*,
    note::{Note, SealedNote},
};

pub type Name = mucodec::String<32>;
pub type Hash = mucodec::Bytes<32>;
pub type PublicKey = mucodec::Bytes<32>;
pub type Signature = mucodec::Bytes<32>;
pub type BlindedValue = mucodec::Bytes<32>;
pub type SecretKey = mucodec::Bytes<32>;
