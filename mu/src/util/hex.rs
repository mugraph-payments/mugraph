use crate::prelude::*;

pub trait FromHex
where
    Self: Sized,
{
    fn from_hex(hex: &str) -> Result<Self>;
}

pub trait ToHex {
    fn to_hex(&self) -> String;
}
