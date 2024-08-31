use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};

use metrics::counter;
use mugraph_core::error::Error;
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use redb::{backends::InMemoryBackend, StorageBackend};

#[derive(Debug)]
pub struct TestBackend {
    pub enable_failures: Arc<AtomicBool>,
    rng: RwLock<ChaCha20Rng>,
    failure_rate: f64,
    inner: InMemoryBackend,
}

impl StorageBackend for TestBackend {
    #[inline]
    fn len(&self) -> Result<u64, std::io::Error> {
        counter!("mugraph.node.database.len_calls").increment(1);

        self.maybe_fail()?;
        self.inner.len()
    }

    #[inline]
    fn read(&self, offset: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
        counter!("mugraph.node.database.read_calls").increment(1);

        self.maybe_fail()?;
        self.inner.read(offset, len)
    }

    #[inline]
    fn set_len(&self, len: u64) -> Result<(), std::io::Error> {
        counter!("mugraph.node.database.set_len_calls").increment(1);

        self.maybe_fail()?;
        self.inner.set_len(len)
    }

    #[inline]
    fn sync_data(&self, eventual: bool) -> Result<(), std::io::Error> {
        counter!("mugraph.node.database.sync_data_calls").increment(1);

        self.maybe_fail()?;
        self.inner.sync_data(eventual)
    }

    #[inline]
    fn write(&self, offset: u64, data: &[u8]) -> Result<(), std::io::Error> {
        counter!("mugraph.node.database.write_calls").increment(1);

        self.maybe_fail()?;
        self.inner.write(offset, data)
    }
}

impl TestBackend {
    pub fn new(rng: ChaCha20Rng, failure_rate: f64) -> Self {
        Self {
            rng: rng.into(),
            failure_rate,
            enable_failures: Arc::new(AtomicBool::new(false)),
            inner: InMemoryBackend::new(),
        }
    }

    #[inline]
    fn maybe_fail(&self) -> Result<(), Error> {
        let mut rng = self.rng.write()?;

        if !self.enable_failures.load(Ordering::Relaxed) {
            return Ok(());
        }

        if rng.gen_range(0f64..self.failure_rate) < self.failure_rate {
            counter!("mugraph.node.database.injected_failures").increment(1);

            Err(Error::StorageError {
                reason: "Triggered failure on database backend".to_string(),
            })
        } else {
            Ok(())
        }
    }
}
