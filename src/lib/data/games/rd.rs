//! Data structures for Rhythm Doctor
use crate::data::game::Game;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RhythmDoctor;

#[typetag::serde(name = "rd")]
impl Game for RhythmDoctor {
    fn pretty_name(&self) -> &'static str {
        "Rhythm Doctor"
    }
    fn url_shortname(&self) -> &'static str {
        "rd"
    }
}

// register_game!(RhythmDoctor); // TODO
