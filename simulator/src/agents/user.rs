use mugraph_client::prelude::*;
use rand::{CryptoRng, RngCore};

use crate::util::Location;

pub struct User {
    pub location: Location,
    pub notes: Vec<Note>,
}

impl User {
    pub fn new<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        Self {
            location: Location::random(&mut rng),
            notes: vec![],
        }
    }
}
