//! Data structures for Guitar Hero III: Legends of Rock.
use crate::game::Game;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GuitarHero3 {}

#[typetag::serde(name = "gh3")]
impl Game for GuitarHero3 {
    fn pretty_name(&self) -> &'static str {
        "Guitar Hero III: Legends of Rock"
    }
    fn url_shortname(&self) -> &'static str {
        "gh3"
    }
}
