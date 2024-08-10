use std::time::Duration;

use rand::{Rng, RngCore};

pub const EARTH_RADIUS: f64 = 6371.0; // km
pub const SPEED_OF_LIGHT: f64 = 299792.458; // km/s

pub struct Location(pub f64, pub f64);

impl Location {
    pub fn random<R: RngCore>(mut rng: R) -> Self {
        Self(rng.gen_range(-90.0..90.0), rng.gen_range(-180.0..180.0))
    }

    pub fn latency_to(&self, other: &Location) -> Duration {
        let lat1 = self.0.to_radians();
        let lon1 = self.1.to_radians();
        let lat2 = other.0.to_radians();
        let lon2 = other.1.to_radians();

        let dlat = lat2 - lat1;
        let dlon = lon2 - lon1;

        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        let distance = EARTH_RADIUS * c;
        let latency_secs = distance / SPEED_OF_LIGHT;

        Duration::from_secs_f64(latency_secs)
    }
}
