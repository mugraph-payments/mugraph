mod bitset;

pub use self::bitset::*;

#[macro_export]
macro_rules! timed {
    ($name:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();

        ::metrics::histogram!("mugraph.task", "name" => $name).record(duration.as_millis_f64());

        result
    }};
}

#[macro_export]
macro_rules! inc {
    ($resource:expr) => {{
        ::metrics::counter!("mugraph.resources", "name" => $resource).increment(1);
    }};
}
