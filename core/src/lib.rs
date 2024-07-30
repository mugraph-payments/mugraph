#![no_std]

pub mod contracts;

mod error;
mod types;

pub use self::{error::*, types::*};

pub const OUTPUT_SEP: Hash = Hash::new([
    251, 27, 10, 119, 219, 137, 49, 221, 246, 211, 108, 158, 213, 143, 56, 34, 184, 84, 252, 192,
    213, 154, 116, 137, 200, 235, 231, 113, 178, 201, 48, 84,
]);
pub const CHANGE_SEP: Hash = Hash::new([
    79, 137, 80, 88, 98, 115, 151, 241, 192, 91, 151, 240, 66, 7, 83, 47, 252, 9, 195, 57, 84, 201,
    158, 76, 251, 117, 116, 203, 34, 242, 57, 247,
]);
pub const HTC_SEP: Hash = Hash::new([
    244, 129, 16, 184, 206, 78, 78, 149, 20, 45, 241, 229, 142, 175, 218, 14, 173, 29, 12, 6, 180,
    108, 3, 238, 41, 141, 212, 239, 112, 242, 238, 62,
]);

pub trait SerializeBytes
where
    Self: Sized,
{
    const SIZE: usize;

    fn to_slice(&self, out: &mut [u8]);
    fn from_slice(input: &[u8]) -> Result<Self>;

    fn digest(&self) -> Hash {
        let mut out = [0u8; 1024];
        self.to_slice(&mut out);

        Hash::digest(&out).unwrap()
    }
}

impl SerializeBytes for u64 {
    const SIZE: usize = size_of::<Self>();

    fn to_slice(&self, out: &mut [u8]) {
        out[..Self::SIZE].copy_from_slice(&self.to_le_bytes())
    }

    fn from_slice(input: &[u8]) -> Result<Self> {
        Ok(Self::from_be_bytes(input[..Self::SIZE].try_into()?))
    }
}

impl<A: SerializeBytes, B: SerializeBytes> SerializeBytes for (A, B) {
    const SIZE: usize = A::SIZE + B::SIZE;

    fn to_slice(&self, out: &mut [u8]) {
        self.0.to_slice(&mut out[..A::SIZE]);
        self.1.to_slice(&mut out[A::SIZE..B::SIZE]);
    }

    fn from_slice(input: &[u8]) -> Result<Self> {
        let a = A::from_slice(&input[..A::SIZE])?;
        let b = B::from_slice(&input[A::SIZE..B::SIZE])?;

        Ok((a, b))
    }
}

impl<const N: usize> SerializeBytes for [u8; N] {
    const SIZE: usize = N;

    fn to_slice(&self, out: &mut [u8]) {
        out[..N].copy_from_slice(self)
    }

    fn from_slice(input: &[u8]) -> Result<Self> {
        Ok(input[..N].try_into()?)
    }
}
