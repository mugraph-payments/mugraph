use crate::{Hash, Result};

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
