use minicbor::Decoder;
use mugraph_core::types::Operation;
use mugraph_core_programs_guest::verify;
use risc0_zkvm::guest::env;

fn main() {
    let mut buf = [0u8; size_of::<Operation>()];
    env::read_slice(&mut buf);
    let mut decoder = Decoder::new(&buf);
    let op = decoder.decode().unwrap();

    verify(&op).unwrap();
}
