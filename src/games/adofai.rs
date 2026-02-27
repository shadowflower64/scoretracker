//! Data structures for A Dance of Fire and Ice
use crate::game::Game;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ADOFAI {}

#[typetag::serde(name = "adofai")]
impl Game for ADOFAI {
    fn pretty_name(&self) -> &'static str {
        "A Dance of Fire and Ice"
    }
    fn url_shortname(&self) -> &'static str {
        "adofai"
    }
}
