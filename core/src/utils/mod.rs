mod bitset;

pub use self::bitset::*;

#[macro_export]
macro_rules! inc {
    ($resource:expr) => {{
        ::metrics::counter!("mugraph.resource", "kind" => $resource).increment(1);
    }};
}
