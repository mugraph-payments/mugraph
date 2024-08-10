use mugraph_client::prelude::*;

use crate::util::Location;

pub struct User {
    pub location: Location,
    pub notes: Vec<Note>,
}
