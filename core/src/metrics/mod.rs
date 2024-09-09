use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use indexmap::IndexMap;
use once_cell::sync::Lazy;

use crate::error::Error;

pub static METRICS: Lazy<Mutex<IndexMap<&'static str, Metric>>> =
    Lazy::new(|| Mutex::new(IndexMap::new()));

pub struct Metric {
    start: Instant,
    pub count: u128,
    pub tps: f64,
    pub p25: Duration,
    pub p50: Duration,
    pub p75: Duration,
    pub p99: Duration,
    pub max: Duration,
}

impl Default for Metric {
    fn default() -> Self {
        Metric {
            start: Instant::now(),
            count: 0,
            tps: 0.0,
            p25: Duration::default(),
            p50: Duration::default(),
            p75: Duration::default(),
            p99: Duration::default(),
            max: Duration::default(),
        }
    }
}

impl Metric {
    pub fn register(name: &'static str) -> Result<(), Error> {
        METRICS.lock()?.insert(name, Metric::default());

        Ok(())
    }

    pub fn increment(name: &'static str, duration: Duration) {
        let mut metrics = METRICS.lock().unwrap();

        let this = match metrics.get_mut(name) {
            Some(v) => v,
            None => {
                metrics.insert(name, Metric::default());
                metrics.get_mut(name).unwrap()
            }
        };

        this.count += 1;
        this.tps = this.count as f64 / this.start.elapsed().as_secs_f64();

        // Update max
        if duration > this.max {
            this.max = duration;
        }

        // Update percentiles using P-square algorithm
        Self::update_percentile(&mut this.p25, 0.25, duration, this.count);
        Self::update_percentile(&mut this.p50, 0.5, duration, this.count);
        Self::update_percentile(&mut this.p75, 0.75, duration, this.count);
        Self::update_percentile(&mut this.p99, 0.99, duration, this.count);
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

    pub fn count(&self) -> u128 {
        self.count
    }

    pub fn p25(&self) -> Duration {
        self.p25
    }

    pub fn p50(&self) -> Duration {
        self.p50
    }

    pub fn p75(&self) -> Duration {
        self.p75
    }

    pub fn p99(&self) -> Duration {
        self.p99
    }

    pub fn max(&self) -> Duration {
        self.max
    }
}
