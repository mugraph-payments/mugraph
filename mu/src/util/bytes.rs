use crate::prelude::*;

pub trait FromBytes
where
    Self: Sized,
{
    fn from_bytes(bytes: &[u8]) -> Result<Self>;
}

pub trait ToBytes {
    type Output: AsRef<[u8]>;

    fn to_bytes(&self) -> Self::Output;

    fn to_bytes_vec(&self) -> Vec<u8> {
        self.to_bytes().as_ref().to_vec()
    }

    fn hash_bytes(&self) -> blake3::Hash {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.to_bytes().as_ref());
        hasher.finalize()
    }

    fn is_zero(&self) -> bool {
        let len = self.to_bytes().as_ref().len();
        self.to_bytes_vec() == vec![0; len]
    }
}
