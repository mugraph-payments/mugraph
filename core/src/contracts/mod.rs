use sha2::{Digest, Sha256};

use crate::{Reader, Result, SerializeBytes, Writer};

pub mod fission;
pub mod fusion;

#[macro_export]
macro_rules! build_contract_alias {
    ($stdin:ty, $stdout:ty, $journal:ty) => {
        pub type Context = $crate::contracts::Context<
            { <$stdin>::SIZE },
            { <$stdout>::SIZE },
            { <$journal>::SIZE },
        >;
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
