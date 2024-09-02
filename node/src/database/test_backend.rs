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
        counter!("mugraph.node.database.backend_calls.len").increment(1);
        timed!("mugraph.node.database.backend_times.len", {
            self.inner.len()
        })
    }

    #[inline]
    fn read(&self, offset: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
        counter!("mugraph.node.database.backend_calls.read").increment(1);
        timed!("mugraph.node.database.backend_times.len", {
            self.inner.read(offset, len)
        })
    }

    #[inline]
    fn set_len(&self, len: u64) -> Result<(), std::io::Error> {
        counter!("mugraph.node.database.backend_calls.set_len").increment(1);
        timed!("mugraph.node.database.backend_times.set_len", {
            self.inner.set_len(len)
        })
    }

    #[inline]
    fn sync_data(&self, eventual: bool) -> Result<(), std::io::Error> {
        counter!("mugraph.node.database.backend_calls.sync_data").increment(1);
        timed!("mugraph.node.database.backend_times.sync_data", {
            self.inner.sync_data(eventual)
        })
    }

    #[inline]
    fn write(&self, offset: u64, data: &[u8]) -> Result<(), std::io::Error> {
        counter!("mugraph.node.database.backend_calls.write").increment(1);
        timed!("mugraph.node.database.backend_times.write", {
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
