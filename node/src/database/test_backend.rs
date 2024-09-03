use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};

use metrics::counter;
use mugraph_core::timed;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use redb::{backends::InMemoryBackend, StorageBackend};
use tracing::info;

#[derive(Debug)]
pub struct TestBackend {
    inner: InMemoryBackend,
    rng: RwLock<ChaCha20Rng>,
    inject_failures: Arc<AtomicBool>,
    failure_rate: f64,
}

impl StorageBackend for TestBackend {
    #[inline]
    fn len(&self) -> Result<u64, std::io::Error> {
        counter!("mugraph.database.len.calls").increment(1);

        self.maybe_fail()?;

        timed!("mugraph.database.len.time_taken", { self.inner.len() })
    }

    #[inline]
    fn read(&self, offset: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
        counter!("mugraph.database.read.calls").increment(1);

        self.maybe_fail()?;

        timed!("mugraph.database.len.time_taken", {
            self.inner.read(offset, len)
        })
    }

    #[inline]
    fn set_len(&self, len: u64) -> Result<(), std::io::Error> {
        counter!("mugraph.database.set_len.calls").increment(1);

        self.maybe_fail()?;

        timed!("mugraph.database.set_len.time_taken", {
            self.inner.set_len(len)
        })
    }

    #[inline]
    fn sync_data(&self, eventual: bool) -> Result<(), std::io::Error> {
        counter!("mugraph.database.sync_data.calls").increment(1);

        self.maybe_fail()?;

        timed!("mugraph.database.sync_data.time_taken", {
            self.inner.sync_data(eventual)
        })
    }

    #[inline]
    fn write(&self, offset: u64, data: &[u8]) -> Result<(), std::io::Error> {
        counter!("mugraph.database.write.calls").increment(1);

        self.maybe_fail()?;

        timed!("mugraph.database.write.time_taken", {
            self.inner.write(offset, data)
        })
    }
}

impl TestBackend {
    pub fn new<R: CryptoRng + Rng>(rng: &mut R) -> Self {
        let mut rng = ChaCha20Rng::seed_from_u64(rng.gen());
        let failure_rate = rng.gen_range(0.0f64..0.5f64);

        info!(
            failure_rate = %format!("{:.2}%", failure_rate * 100.0),
            "Starting test database backend"
        );

        Self {
            inner: InMemoryBackend::new(),
            rng: rng.into(),
            inject_failures: AtomicBool::new(false).into(),
            failure_rate,
        }
    }

    #[inline]
    fn maybe_fail(&self) -> Result<(), std::io::Error> {
        counter!("mugraph.database.actions").increment(1);
        let mut rng = self.rng.write().expect("Failed to get rng");

        if !self.inject_failures.load(Ordering::Relaxed) {
            return Ok(());
        }

        if rng.gen_range(0f64..self.failure_rate) < self.failure_rate {
            counter!("mugraph.database.injected_failures").increment(1);

            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Simulated failure",
            ))
        } else {
            Ok(())
        }
    }
}
