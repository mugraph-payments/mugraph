use rand::{prelude::StdRng, SeedableRng};

pub struct Config {
    pub seed: Option<u64>,
    pub user_count: usize,
    pub asset_count: usize,
    pub max_notes_per_user: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            seed: None,
            user_count: 16,
            asset_count: 4,
            max_notes_per_user: 4,
        }
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
