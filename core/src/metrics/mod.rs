use std::{
    collections::BTreeMap,
    sync::RwLock,
    time::{Duration, Instant},
};

use once_cell::sync::Lazy;

use crate::error::Error;

pub static METRICS: Lazy<RwLock<BTreeMap<&'static str, Metric>>> = Lazy::new(Default::default);

#[derive(Clone, Copy)]
pub struct Metric {
    start: Instant,
    pub count: u128,
    pub tps: f64,
    pub p50: Duration,
    pub p99: Duration,
    pub max: Duration,
}

impl Default for Metric {
    fn default() -> Self {
        Metric {
            start: Instant::now(),
            count: 0,
            tps: 0.0,
            p50: Duration::default(),
            p99: Duration::default(),
            max: Duration::default(),
        }
    }
}

impl Metric {
    pub fn register(name: &'static str) -> Result<(), Error> {
        METRICS.write()?.insert(name, Metric::default());
        Ok(())
    }

    pub fn increment(name: &'static str, duration: Duration) {
        let mut metric = METRICS
            .write()
            .unwrap()
            .get(name)
            .copied()
            .unwrap_or_default();

        metric.count += 1;
        metric.tps = metric.count as f64 / metric.start.elapsed().as_secs() as f64;

        // Update max
        if duration > metric.max {
            metric.max = duration;
        }

        // Update percentiles using P-square algorithm
        let count = metric.count;

        Self::update_percentile(&mut metric.p50, 0.5, duration, count);
        Self::update_percentile(&mut metric.p99, 0.99, duration, count);

        METRICS.write().unwrap().insert(name, metric);
    }

    fn update_percentile(
        estimate: &mut Duration,
        percentile: f64,
        new_value: Duration,
        count: u128,
    ) {
        if count == 1 {
            *estimate = new_value;
        } else {
            let sign = if new_value < *estimate { -1.0 } else { 1.0 };
            let delta = sign * (*estimate).as_nanos() as f64 / count as f64;
            let new_estimate = (*estimate).as_nanos() as f64 + delta / percentile;
            *estimate = Duration::from_nanos(new_estimate as u64);
        }
    }
}
