//! Data structures for UNBEATABLE
use crate::data::game::Game;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UNBEATABLE {}

#[typetag::serde(name = "unbeatable")]
impl Game for UNBEATABLE {
    fn pretty_name(&self) -> &'static str {
        "UNBEATABLE"
    }
    fn url_shortname(&self) -> &'static str {
        "unbeatable"
    }
}
