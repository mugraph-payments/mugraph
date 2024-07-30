use crate::{Result, SerializeBytes};

pub mod fission;
pub mod fusion;

pub struct Context<const STDIN: usize, const STDOUT: usize, const JOURNAL: usize> {
    pub stdin: [u8; STDIN],
    pub stdout: [u8; STDOUT],
    pub journal: [u8; JOURNAL],

    pub stdin_offset: usize,
    pub stdout_offset: usize,
    pub journal_offset: usize,
}

impl<const STDIN: usize, const STDOUT: usize, const JOURNAL: usize>
    Context<STDIN, STDOUT, JOURNAL>
{
    pub fn new() -> Self {
        Self {
            stdin: [0; STDIN],
            stdout: [0; STDOUT],
            journal: [0; JOURNAL],
            stdin_offset: 0,
            stdout_offset: 0,
            journal_offset: 0,
        }
    }

    pub fn read_stdin<T: SerializeBytes>(&mut self) -> Result<T> {
        assert!(self.stdin_offset + STDIN >= T::SIZE);

        let result = T::from_slice(&self.stdin[self.stdin_offset..T::SIZE])?;

        self.stdin_offset += T::SIZE;

        Ok(result)
    }

    pub fn write_stdout<T: SerializeBytes>(&mut self, value: &T) {
        assert!(self.stdout_offset + STDOUT >= T::SIZE);

        value.to_slice(&mut self.stdout[self.stdout_offset..self.stdout_offset + T::SIZE]);

        self.stdout_offset += T::SIZE;
    }

    pub fn write_journal<T: SerializeBytes>(&mut self, value: &T) {
        assert!(self.journal_offset + JOURNAL >= T::SIZE);

        value.to_slice(&mut self.journal[self.journal_offset..self.journal_offset + T::SIZE]);

        self.journal_offset += T::SIZE;
    }
}
