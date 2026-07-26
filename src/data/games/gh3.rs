//! Data structures for Guitar Hero III: Legends of Rock.
use crate::data::game::Game;
use serde::{Deserialize, Serialize};

/// A playable part in the chart.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Instrument {
    LeadGuitar,
    RhythmGuitar,
    BassGuitar,
}

/// Difficulty that the performance was played on.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

/// Game mode.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Career,
    Quickplay,
    CoOpCareer,
    FaceOff,
    ProFaceOff,
    Battle,
    CoOp,
    Practice,
}

impl Mode {
    pub fn player_count(&self) -> u8 {
        match self {
            Self::Career | Self::Quickplay | Self::Practice => 1,
            Self::CoOpCareer | Self::FaceOff | Self::ProFaceOff | Self::Battle | Self::CoOp => 2,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Performance {
    ///// Who played what and when. /////
    /// Played instrument.
    pub instrument: Instrument,

    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    ///// Score and stats. /////
    /// Amount of points at the end of the performance.
    pub score: u64,

    /// How many notes were hit successfully.
    pub notes_hit: u64,

    /// The maximum streak achieved during the performance.
    pub max_streak: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Match {
    ///// Who played what and when. /////
    /// Game mode that this match was played on.
    pub mode: Mode,

    ///// Score and stats. /////
    /// Amount of points at the end of the match.
    pub score: u64,

    /// How many notes were hit successfully.
    pub notes_hit: u64,

    /// The maximum streak achieved during the match.
    pub max_streak: u64,

    ///// Game settings and information. /////
    /// String of the game version that was played on for this match.
    pub game_version: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GuitarHero3;

#[typetag::serde(name = "gh3")]
impl Game for GuitarHero3 {
    fn pretty_name(&self) -> &'static str {
        "Guitar Hero III: Legends of Rock"
    }
    fn url_shortname(&self) -> &'static str {
        "gh3"
    }
}
