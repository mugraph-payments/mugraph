use metrics::counter;
use mugraph_core::error::Error;
use rand::Rng;
use rand_chacha::ChaCha20Rng;
use redb::{backends::FileBackend, StorageBackend};
use tracing::error;

#[derive(Debug)]
pub struct TestBackend {
    rng: ChaCha20Rng,
    failure_rate: f64,
    inner: FileBackend,
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
        let file = tempfile::tempfile().unwrap();

        Self {
            rng,
            failure_rate,
            inner: FileBackend::new(file).unwrap(),
        }
    }

    #[inline]
    fn maybe_fail(&self) -> Result<(), Error> {
        if self.rng.clone().gen_bool(self.failure_rate) {
            counter!("mugraph.node.database.injected_failures").increment(1);

            error!(failure_rate = self.failure_rate, "Storage call failed");

            Err(Error::StorageError {
                reason: "Triggered failure on database backend".to_string(),
            })
        } else {
            Ok(())
        }
    }
}
