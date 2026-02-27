//! Data structures for Clone Hero
use crate::game::Game;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CloneHero {}

#[typetag::serde(name = "ch")]
impl Game for CloneHero {
    fn pretty_name(&self) -> &'static str {
        "Clone Hero"
    }
    fn url_shortname(&self) -> &'static str {
        "ch"
    }
}
