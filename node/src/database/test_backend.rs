use std::{fs::OpenOptions, path::PathBuf};

use metrics::counter;
use mugraph_core::error::Error;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use redb::{backends::FileBackend, StorageBackend};
use tempfile::NamedTempFile;
use tracing::info;

#[derive(Debug)]
pub struct TestBackend {
    inner: FileBackend,
    rng: ChaCha20Rng,
    pub path: PathBuf,
    failure_rate: f64,
}

impl StorageBackend for TestBackend {
    #[inline]
    #[tracing::instrument(skip_all)]
    fn len(&self) -> Result<u64, std::io::Error> {
        self.maybe_fail()?;
        self.inner.len()
    }

    #[inline]
    #[tracing::instrument(skip_all)]
    fn read(&self, offset: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
        self.maybe_fail()?;
        self.inner.read(offset, len)
    }

    #[inline]
    #[tracing::instrument(skip_all)]
    fn set_len(&self, len: u64) -> Result<(), std::io::Error> {
        self.maybe_fail()?;
        self.inner.set_len(len)
    }

    #[inline]
    #[tracing::instrument(skip_all)]
    fn sync_data(&self, eventual: bool) -> Result<(), std::io::Error> {
        self.maybe_fail()?;
        self.inner.sync_data(eventual)
    }

    #[inline]
    #[tracing::instrument(skip_all)]
    fn write(&self, offset: u64, data: &[u8]) -> Result<(), std::io::Error> {
        self.maybe_fail()?;
        self.inner.write(offset, data)
    }
}

impl TestBackend {
    pub fn new<R: CryptoRng + Rng>(rng: &mut R, path: Option<PathBuf>) -> Result<Self, Error> {
        let mut rng = ChaCha20Rng::seed_from_u64(rng.gen());
        let failure_rate = rng.gen_range(0.0f64..0.01f64);

        info!(
            failure_rate = %format!("{:.2}%", failure_rate * 100.0),
            "Starting test database backend"
        );

        let tmp = path.unwrap_or(NamedTempFile::new()?.path().into());
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&tmp)?;

        Ok(Self {
            inner: FileBackend::new(file)?,
            rng,
            failure_rate,
            path: tmp,
        })
    }

    #[inline]
    fn maybe_fail(&self) -> Result<(), std::io::Error> {
        let mut rng = self.rng.clone();

        if rng.gen_bool(self.failure_rate) {
            counter!("mugraph.simulator.injected_failures").increment(1);

            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "injected_error",
            ))
        } else {
            Ok(())
        }
    }
}
