//! Data structures for Guitar Hero III: Legends of Rock.
//!
//! Progress status: All fields from the original spreadsheet are implemented.

use crate::data::game::IncompleteOrCritical::Incomplete;
use crate::data::game::{Game, ImportMatchResult, ImportSongResult, SkipOrQuit, SpreadsheetContext};
use crate::data::scoreboard::r#match::{CommonMatchInfo, MatchTrait};
use crate::data::scoreboard::performance::{CommonPerformanceInfo, PerformanceTrait};
use crate::spreadsheet::{RecordError, record::Record};
use crate::util::command_line::AskError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Game mode.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Career,
    Quickplay,
    UnknownSingle, // Either Career or Quickplay mode
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
            Self::Career | Self::Quickplay | Self::Practice | Self::UnknownSingle => 1,
            Self::CoOpCareer | Self::FaceOff | Self::ProFaceOff | Self::Battle | Self::CoOp => 2,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Match {
    #[serde(flatten)]
    pub common: CommonMatchInfo,

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
    /// None for unknown.
    pub game_version: Option<String>,
}

#[typetag::serde(name = "gh3")]
impl MatchTrait for Match {
    fn common(&self) -> &CommonMatchInfo {
        &self.common
    }
    fn sorting_key(&self) -> f64 {
        todo!()
    }
}

/// A playable part in the chart.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Instrument {
    LeadGuitar,
    RhythmGuitar,
    BassGuitar,
}

#[derive(Error, Debug)]
#[error("not an instrument: {0}")]
pub struct NotAnInstrument(String);

impl From<NotAnInstrument> for RecordError {
    fn from(value: NotAnInstrument) -> Self {
        Self::Custom(Box::new(value))
    }
}

impl TryFrom<&str> for Instrument {
    type Error = NotAnInstrument;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "guitar" => Ok(Self::LeadGuitar),
            "rhythm" => Ok(Self::RhythmGuitar),
            "bass" => Ok(Self::BassGuitar),
            a => Err(NotAnInstrument(a.to_owned())),
        }
    }
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

#[derive(Error, Debug)]
#[error("not a difficulty: {0}")]
pub struct NotADifficulty(String);

impl From<NotADifficulty> for RecordError {
    fn from(value: NotADifficulty) -> Self {
        Self::Custom(Box::new(value))
    }
}

impl TryFrom<&str> for Difficulty {
    type Error = NotADifficulty;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "easy" => Ok(Self::Easy),
            "medium" => Ok(Self::Medium),
            "hard" => Ok(Self::Hard),
            "expert" => Ok(Self::Expert),
            a => Err(NotADifficulty(a.to_owned())),
        }
    }
}

/// Clear type.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Lamp {
    None,
    Clear,
    FC,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Performance {
    #[serde(flatten)]
    pub common: CommonPerformanceInfo,

    ///// Who played what and when. /////
    /// Played instrument.
    pub instrument: Instrument,

    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    ///// Score and stats. /////
    /// Clear type.
    pub lamp: Lamp,

    /// Amount of points at the end of the performance.
    pub score: u64,

    /// How many notes were hit successfully.
    pub notes_hit: u64,

    /// How many notes were in the chart (TODO: should be const across different scores of the chart)
    pub notes_total: u64,

    /// The maximum streak achieved during the performance.
    pub max_streak: u64,
}

#[typetag::serde(name = "gh3")]
impl PerformanceTrait for Performance {
    fn common(&self) -> &CommonPerformanceInfo {
        &self.common
    }
    fn sorting_key(&self) -> f64 {
        self.score as f64
    }
    fn ask_for_performance_edit(&mut self) -> Result<(), AskError> {
        todo!()
    }
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

    fn create_match_and_performance_from_spreadsheet_record(&self, record: &Record, ctx: &mut SpreadsheetContext) -> ImportMatchResult {
        // println!("{record}");
        let mut lamp = Lamp::None;
        if record.bool("clear")? {
            lamp = Lamp::Clear;
        }
        if record.bool("fc")? {
            lamp = Lamp::FC;
        }
        let performance_data = Performance {
            common: ctx.create_common_p(record)?,
            instrument: record.string_enum("instrument")?,
            difficulty: record.string_enum("difficulty")?,
            lamp,
            score: record.int("score").or_skip()?,
            notes_hit: record.int("hit_notes")?,
            notes_total: record.int("total_notes")?,
            max_streak: record.int("note_streak")?,
        };
        let match_data = Match {
            common: ctx.create_common_m(record, &[&performance_data])?,
            mode: Mode::UnknownSingle,
            score: record.int("score")?,
            notes_hit: record.int("hit_notes")?,
            max_streak: record.int("note_streak")?,
            game_version: None,
        };
        Ok((Box::new(match_data), vec![Box::new(performance_data)]))
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut SpreadsheetContext) -> ImportSongResult {
        Err(Incomplete(RecordError::NotImplemented))
    }
}
