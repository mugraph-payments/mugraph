use iai::black_box;
use mugraph_core::types::*;
use mugraph_core_programs_guest::verify;

fn bench_consume() {
    let op = Operation::Consume {
        input: Sealed {
            parent: [1u8; 32].into(),
            index: 0,
            data: Note {
                asset_id: [2u8; 32].into(),
                amount: 1337,
                program_id: None,
                datum: None,
            },
        },
        output: Note {
            asset_id: [2u8; 32].into(),
            amount: 1337,
            program_id: None,
            datum: None,
        },
    };

    verify(&black_box(op)).unwrap();
}

iai::main!(main);
