#![no_std]
#![feature(test)]
extern crate test;

use core::ops::Deref;

use test::Bencher;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Hash([u8; 32]);

impl Deref for Hash {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[bench]
fn bench_validate(b: &mut Bencher) {
    let transaction = Transaction {
        manifest: Manifest {
            programs: ProgramSet {
                validate: VALIDATE_ID.into(),
            },
        },
        inputs: Inputs {
            parents: [[0; 32].into(); 4],
            indexes: [0, 1, 2, 3],
            asset_ids: [1, 2, 3, 1],
            amounts: [100, 200, 300, 400],
            program_id: [[0; 32].into(); 4],
            data: [u32::MAX; 4],
        },
        outputs: Outputs {
            asset_ids: [1, 2, 3, 1],
            amounts: [150, 200, 300, 350],
            program_id: [[0; 32].into(); 4],
            data: [u32::MAX; 4],
        },
        data: [0; 256 * 8],
        assets: [
            [1; 32].into(),
            [2; 32].into(),
            [3; 32].into(),
            [0; 32].into(),
        ],
    };

    b.iter(|| validate(&transaction));
}
