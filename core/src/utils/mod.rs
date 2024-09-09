mod bitset;

use metrics::{describe_histogram, Unit};
pub use mugraph_macros::timed;

pub use self::bitset::*;

pub fn describe_metrics() {
    describe_histogram!(
        "mugraph.task.durations",
        Unit::Nanoseconds,
        "Duration of a task"
    );
}

#[macro_export]
macro_rules! inc {
    ($resource:expr) => {{
        ::metrics::counter!("mugraph.resource", "kind" => $resource).increment(1);
    }};
}
