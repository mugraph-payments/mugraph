use proptest::prelude::*;
use rand::{prelude::*, rngs::StdRng};

pub fn rng() -> impl Strategy<Value = StdRng> {
    any::<[u8; 32]>().prop_map(StdRng::from_seed)
}
