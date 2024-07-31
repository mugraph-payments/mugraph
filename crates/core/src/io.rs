use sha2::{Digest, Sha256};

use crate::{Result, SerializeBytes};

pub struct Reader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn read<T: SerializeBytes>(&mut self) -> Result<T> {
        assert!(self.offset + T::SIZE <= self.data.len());

        let result = T::from_slice(&self.data[self.offset..self.offset + T::SIZE])?;
        self.offset += T::SIZE;

        Ok(result)
    }
}

pub struct Writer<'a> {
    data: &'a mut [u8],
    offset: usize,
}

impl<'a> Writer<'a> {
    pub fn new(data: &'a mut [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn write<T: SerializeBytes>(&mut self, value: &T) {
        assert!(self.offset + T::SIZE <= self.data.len());

        value.to_slice(&mut self.data[self.offset..T::SIZE]);
        self.offset += T::SIZE;
    }
}

#[macro_export]
macro_rules! build_context_alias {
    ($stdin:ty, $stdout:ty, $journal:ty) => {
        pub type Context =
            $crate::Context<{ <$stdin>::SIZE }, { <$stdout>::SIZE }, { <$journal>::SIZE }>;
    };
}

pub struct Context<const STDIN: usize, const STDOUT: usize, const JOURNAL: usize> {
    pub hasher: Sha256,
    pub stdin: [u8; STDIN],
    pub stdout: [u8; STDOUT],
    pub journal: [u8; JOURNAL],
}

impl<const STDIN: usize, const STDOUT: usize, const JOURNAL: usize>
    Context<STDIN, STDOUT, JOURNAL>
{
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
            stdin: [0; STDIN],
            stdout: [0; STDOUT],
            journal: [0; JOURNAL],
        }
    }

    pub fn read_stdin<T: SerializeBytes>(&mut self) -> Result<T> {
        let mut r = Reader::new(&self.stdin);
        r.read()
    }

    pub fn write_stdin<T: SerializeBytes>(&mut self, value: &T) {
        let mut w = Writer::new(&mut self.stdin);
        w.write(value);
    }

    pub fn write_stdout<T: SerializeBytes>(&mut self, value: &T) {
        let mut w = Writer::new(&mut self.stdout);
        w.write(value);
    }

    pub fn write_journal<T: SerializeBytes>(&mut self, value: &T) {
        let mut w = Writer::new(&mut self.journal);
        w.write(value);
    }
}

impl<const STDIN: usize, const STDOUT: usize, const JOURNAL: usize> Default
    for Context<STDIN, STDOUT, JOURNAL>
{
    fn default() -> Self {
        Self::new()
    }
}
