use clap::Parser;
use rand::{prelude::StdRng, SeedableRng};

#[derive(Debug, Clone, Copy, Parser)]
#[command(version, author, about)]
pub struct Config {
    #[clap(short, long, env = "MUGRAPH_SIMULATOR_SEED")]
    pub seed: Option<u64>,
    #[clap(
        short,
        long = "users",
        default_value = "128",
        env = "MUGRAPH_SIMULATOR_USERS"
    )]
    pub users: usize,
    #[clap(
        short,
        long = "assets",
        default_value = "16",
        env = "MUGRAPH_SIMULATOR_ASSETS"
    )]
    pub assets: usize,
    #[clap(
        short,
        long = "notes",
        default_value = "16",
        env = "MUGRAPH_SIMULATOR_NOTES_PER_USER"
    )]
    pub notes: usize,
    #[clap(long = "steps", env = "MUGRAPH_SIMULATOR_STEPS")]
    pub steps: Option<u64>,
    #[clap(
        short,
        long = "threads",
        env = "MUGRAPH_SIMULATOR_THREADS",
        default_value_t = num_cpus::get_physical()
    )]
    pub threads: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self::parse()
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rng(&self) -> StdRng {
        match self.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        }
    }
}
