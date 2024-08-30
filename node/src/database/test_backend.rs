use mugraph_core::error::Error;
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use redb::{backends::InMemoryBackend, StorageBackend};

#[derive(Debug)]
pub struct TestBackend {
    rng: ChaCha20Rng,
    failure_ratio: f64,
    inner: InMemoryBackend,
}

impl StorageBackend for TestBackend {
    #[inline]
    fn len(&self) -> Result<u64, std::io::Error> {
        self.maybe_fail()?;
        self.inner.len()
    }

    #[inline]
    fn read(&self, offset: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
        self.maybe_fail()?;
        self.inner.read(offset, len)
    }

    #[inline]
    fn set_len(&self, len: u64) -> Result<(), std::io::Error> {
        self.maybe_fail()?;
        self.inner.set_len(len)
    }

    #[inline]
    fn sync_data(&self, eventual: bool) -> Result<(), std::io::Error> {
        self.maybe_fail()?;
        self.inner.sync_data(eventual)
    }

    #[inline]
    fn write(&self, offset: u64, data: &[u8]) -> Result<(), std::io::Error> {
        self.maybe_fail()?;
        self.inner.write(offset, data)
    }
}

impl TestBackend {
    pub fn new(rng: ChaCha20Rng, failure_ratio: f64) -> Self {
        Self {
            rng,
            failure_ratio,
            inner: InMemoryBackend::new(),
        }
    }

    #[inline]
    fn maybe_fail(&self) -> Result<(), Error> {
        let mut rng = self.rng.clone();
        let chance = rng.gen_range(0f64..1f64);

        if chance < self.failure_ratio {
            Err(Error::StorageError {
                reason: "Triggered failure on database backend".to_string(),
            })
        } else {
            Ok(())
        }
    }
}
