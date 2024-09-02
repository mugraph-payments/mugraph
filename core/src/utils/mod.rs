mod bitset;

pub use self::bitset::*;

#[macro_export]
macro_rules! timed {
    ($histogram:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();

        ::metrics::histogram!($histogram).record(duration.as_millis_f64());
        result
    }};
}
