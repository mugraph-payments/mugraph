use std::{
    fs::OpenOptions,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use metrics::counter;
use mugraph_core::{error::Error, timed};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use redb::{backends::FileBackend, StorageBackend};
use tempfile::NamedTempFile;
use tracing::info;

#[derive(Debug)]
pub struct TestBackend {
    inner: FileBackend,
    rng: ChaCha20Rng,
    pub inject_failures: Arc<AtomicBool>,
    pub path: PathBuf,
    failure_rate: f64,
}

impl StorageBackend for TestBackend {
    #[inline]
    fn len(&self) -> Result<u64, std::io::Error> {
        self.maybe_fail()?;
        timed!("database.len", { self.inner.len() })
    }

    #[inline]
    fn read(&self, offset: u64, len: usize) -> Result<Vec<u8>, std::io::Error> {
        self.maybe_fail()?;
        timed!("database.len", { self.inner.read(offset, len) })
    }

    #[inline]
    fn set_len(&self, len: u64) -> Result<(), std::io::Error> {
        self.maybe_fail()?;
        timed!("database.set_len", { self.inner.set_len(len) })
    }

    #[inline]
    fn sync_data(&self, eventual: bool) -> Result<(), std::io::Error> {
        self.maybe_fail()?;
        timed!("database.sync_data", { self.inner.sync_data(eventual) })
    }

    #[inline]
    fn write(&self, offset: u64, data: &[u8]) -> Result<(), std::io::Error> {
        self.maybe_fail()?;
        timed!("database.write", { self.inner.write(offset, data) })
    }
}

impl TestBackend {
    pub fn new<R: CryptoRng + Rng>(
        rng: &mut R,
        path: Option<PathBuf>,
    ) -> Result<(Arc<AtomicBool>, Self), Error> {
        let mut rng = ChaCha20Rng::seed_from_u64(rng.gen());
        let failure_rate = rng.gen_range(0.2f64..0.99f64);

        info!(
            failure_rate = %format!("{:.2}%", failure_rate * 100.0),
            "Starting test database backend"
        );

        let tmp = path.unwrap_or(NamedTempFile::new()?.path().into());
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(tmp.clone())?;

        let inject_failures: Arc<_> = AtomicBool::new(false).into();

        Ok((
            inject_failures.clone(),
            Self {
                inner: FileBackend::new(file)?,
                rng,
                inject_failures,
                failure_rate,
                path: tmp,
            },
        ))
    }

    #[inline]
    fn maybe_fail(&self) -> Result<(), std::io::Error> {
        let mut rng = self.rng.clone();

        if !self.inject_failures.load(Ordering::Relaxed) {
            return Ok(());
        }

        if rng.gen_range(0.0..1.0) < self.failure_rate {
            counter!("injected_failures").increment(1);

            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "injected_error",
            ))
        } else {
            Ok(())
        }
    }
}
