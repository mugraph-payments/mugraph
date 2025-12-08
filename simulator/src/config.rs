use clap::Parser;
use rand::{SeedableRng, prelude::*, rng};
use rand_chacha::ChaChaRng;
use tracing::info;

#[derive(Debug, Clone, Parser)]
#[command(version, author, about)]
pub struct Config {
    #[clap(long, env = "MUGRAPH_SIMULATOR_SEED")]
    /// The seed to use for the simulation
    pub seed: Option<u64>,

    #[clap(
        long = "assets",
        default_value = "4",
        env = "MUGRAPH_SIMULATOR_ASSETS"
    )]
    /// The amount of assets to simulate
    pub assets: usize,

    #[clap(
        long = "notes",
        default_value = "256",
        env = "MUGRAPH_SIMULATOR_NOTES"
    )]
    /// The maximum amount of notes each user should have at simulation start
    pub notes: usize,

    #[clap(long, default_value = "http://localhost:9999")]
    /// The address of the running node (must start with http:// or https://)
    pub node_addr: String,
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

    pub fn rng(&self) -> ChaChaRng {
        let seed = match self.seed {
            Some(seed) => seed,
            None => rng().random(),
        };

        info!(
            seed = %seed,
            was_provided = self.seed.is_some(),
            "Initializing main RNG with seed"
        );

        ChaChaRng::seed_from_u64(seed)
    }
}
