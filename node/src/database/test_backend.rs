use metrics::counter;
use mugraph_core::timed;
use redb::{backends::InMemoryBackend, StorageBackend};

#[derive(Debug)]
pub struct TestBackend {
    inner: InMemoryBackend,
}

impl StorageBackend for TestBackend {
    #[inline]
    fn len(&self) -> Result<u64, std::io::Error> {
        counter!("mugraph.database.len.calls").increment(1);
        timed!("mugraph.database.len.time_taken", { self.inner.len() })
    }

    #[inline]
    fn read(&self, offset: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
        counter!("mugraph.database.read.calls").increment(1);
        timed!("mugraph.database.len.time_taken", {
            self.inner.read(offset, len)
        })
    }

    #[inline]
    fn set_len(&self, len: u64) -> Result<(), std::io::Error> {
        counter!("mugraph.database.set_len.calls").increment(1);
        timed!("mugraph.database.set_len.time_taken", {
            self.inner.set_len(len)
        })
    }

    #[inline]
    fn sync_data(&self, eventual: bool) -> Result<(), std::io::Error> {
        counter!("mugraph.database.sync_data.calls").increment(1);

        timed!("mugraph.database.sync_data.time_taken", {
            self.inner.sync_data(eventual)
        })
    }

    #[inline]
    fn write(&self, offset: u64, data: &[u8]) -> Result<(), std::io::Error> {
        counter!("mugraph.database.write.calls").increment(1);

        timed!("mugraph.database.write.time_taken", {
            self.inner.write(offset, data)
        })
    }
}

impl Default for TestBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl TestBackend {
    pub fn new() -> Self {
        Self {
            inner: InMemoryBackend::new(),
        }
    }
}
