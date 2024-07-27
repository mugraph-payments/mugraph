mod circuits {
    include!(concat!(env!("OUT_DIR"), "/methods.rs"));
}

pub use circuits::MUGRAPH_CIRCUITS_SWAP_GUEST_ELF as ELF;
pub use circuits::MUGRAPH_CIRCUITS_SWAP_GUEST_ID as ID;
pub use circuits::MUGRAPH_CIRCUITS_SWAP_GUEST_PATH as PATH;
