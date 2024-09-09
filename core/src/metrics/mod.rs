use std::{
    sync::RwLock,
    time::{Duration, Instant},
};

use indexmap::IndexMap;
use once_cell::sync::Lazy;

use crate::error::Error;

pub static REGISTERED_METRICS: Lazy<RwLock<IndexMap<&'static str, u32>>> =
    Lazy::new(|| RwLock::new(IndexMap::new()));
pub static METRICS: Lazy<RwLock<Vec<Metric>>> = Lazy::new(|| RwLock::new(vec![]));

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
        let mut m = METRICS.write()?;
        let mut rm = REGISTERED_METRICS.write()?;

        rm.insert(name, m.len() as u32);
        m.push(Metric::default());

        Ok(())
    }

    pub fn increment(name: &'static str, duration: Duration) {
        let i = {
            let rm = REGISTERED_METRICS.read().unwrap();
            rm.get(name).copied()
        };

        match i {
            Some(i) => {
                let mut metrics = METRICS.write().unwrap();
                let metric = &mut metrics[i as usize];

                metric.count += 1;
                metric.tps = metric.count as f64 / metric.start.elapsed().as_secs_f64();

                // Update max
                if duration > metric.max {
                    metric.max = duration;
                }

                // Update percentiles using P-square algorithm
                let count = metric.count;

                Self::update_percentile(&mut metric.p50, 0.5, duration, count);
                Self::update_percentile(&mut metric.p99, 0.99, duration, count);
            }
            None => {
                Self::register(name).unwrap();
                Self::increment(name, duration);
            }
        }
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
