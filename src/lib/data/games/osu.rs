//! Data structures for osu!
use crate::data::game::Game;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Osu;

#[typetag::serde(name = "osu")]
impl Game for Osu {
    fn pretty_name(&self) -> &'static str {
        "osu!"
    }
    fn url_shortname(&self) -> &'static str {
        "osu"
    }
}

// register_game!(Osu); // TODO
