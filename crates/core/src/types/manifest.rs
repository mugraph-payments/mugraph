use crate::types::Hash;

pub struct Manifest {
    pub id: Hash,
    pub elf: Vec<u8>,
}
