use crate::*;

pub trait SerializeBytes
where
    Self: Sized,
{
    const SIZE: usize;

    fn to_slice(&self, out: &mut [u8]);
    fn from_slice(input: &[u8]) -> Result<Self>;
}

impl SerializeBytes for u64 {
    const SIZE: usize = size_of::<Self>();

    #[inline]
    fn to_slice(&self, out: &mut [u8]) {
        assert_eq!(out.len(), Self::SIZE);
        out[..Self::SIZE].copy_from_slice(&self.to_le_bytes())
    }

    #[inline]
    fn from_slice(input: &[u8]) -> Result<Self> {
        assert_eq!(input.len(), Self::SIZE);
        Ok(Self::from_le_bytes(input[..Self::SIZE].try_into()?))
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

impl<A: SerializeBytes, B: SerializeBytes> SerializeBytes for (A, B) {
    const SIZE: usize = A::SIZE + B::SIZE;

    #[inline]
    fn to_slice(&self, out: &mut [u8]) {
        let mut w = Writer::new(out);
        w.write(&self.0);
        w.write(&self.1);
    }

    #[inline]
    fn from_slice(input: &[u8]) -> Result<Self> {
        let mut r = Reader::new(input);

        Ok((r.read()?, r.read()?))
    }
}
