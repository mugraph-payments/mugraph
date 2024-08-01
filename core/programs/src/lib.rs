#[doc(hidden)]
pub mod __build {
    include!(concat!(env!("OUT_DIR"), "/methods.rs"));
}
