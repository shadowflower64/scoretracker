//! Data structures for Rock Band 4.
//!
//! Progress status: All fields from the original spreadsheet are implemented.

use crate::data::game::Game;
use crate::data::scoreboard::r#match::MatchTrait;
use crate::data::scoreboard::performance::PerformanceTrait;
use crate::data::scoreboard::{r#match::CommonMatchInfo, performance::CommonPerformanceInfo};
use crate::spreadsheet::IncompleteOrCritical::Continue;
use crate::spreadsheet::context::Context;
use crate::spreadsheet::{BadRecordError, ParseMatchRecordResult, ParseSongRecordResult, SkipOrQuit};
use crate::{spreadsheet::record::Record, util::command_line::AskError};
use serde::{Deserialize, Serialize};

/// Game mode.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Quickplay,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Match {
    #[serde(flatten)]
    pub common: CommonMatchInfo,

    /// Game mode that this match was played on.
    pub mode: Mode,

    /// String of the game version that was played on for this match.
    /// None for unknown.
    pub game_version: Option<String>,
}

#[typetag::serde(name = "rb4")]
impl MatchTrait for Match {
    fn common(&self) -> &CommonMatchInfo {
        &self.common
    }
    fn sorting_key(&self) -> f64 {
        todo!()
    }
}

/// A playable part in the chart.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Instrument {
    Guitar,
    Bass,
    Drums,
    Keys,
    Vocals,
    ProDrums,
    Harmonies,
}

impl Instrument {
    pub fn is_pro(&self) -> bool {
        matches!(&self, Self::ProDrums | Self::Harmonies)
    }
}

impl TryFrom<&str> for Instrument {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "guitar" => Ok(Self::Guitar),
            "bass" => Ok(Self::Bass),
            "drums" => Ok(Self::Drums),
            "keys" => Ok(Self::Keys),
            "vocals" => Ok(Self::Vocals),
            "pro_drums" => Ok(Self::ProDrums),
            "pro_vocals" => Ok(Self::Harmonies),
            _ => Err("rb4::Instrument"),
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
    Brutal, // TODO: confirm that brutal mode does not allow for chart difficulties other than Expert
}

impl TryFrom<&str> for Difficulty {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "easy" => Ok(Self::Easy),
            "medium" => Ok(Self::Medium),
            "hard" => Ok(Self::Hard),
            "expert" => Ok(Self::Expert),
            "brutal" => Ok(Self::Brutal),
            _ => Err("rb4::Difficulty"),
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

    /// Played instrument.
    pub instrument: Instrument,

    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    /// Clear type.
    pub lamp: Lamp,

    /// Amount of points at the end of the performance.
    pub score: u64,

    /// How many notes were hit successfully.
    pub notes_hit: u32,

    /// How many notes were in the chart (TODO: should be const across different scores of the chart)
    pub notes_total: u32,

    /// The maximum streak achieved during the performance.
    pub max_streak: u32,
}

#[typetag::serde(name = "rb4")]
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
pub struct RockBand4;

#[typetag::serde(name = "rb4")]
impl Game for RockBand4 {
    fn pretty_name(&self) -> &'static str {
        "Rock Band 4"
    }
    fn url_shortname(&self) -> &'static str {
        "rb4"
    }

    fn create_match_and_performance_from_spreadsheet_record(&self, record: &Record, ctx: &mut Context) -> ParseMatchRecordResult {
        // println!("{record}");
        ctx.check_early_skip(record)?;
        let mut lamp = Lamp::None;
        if record.bool("clear")? {
            lamp = Lamp::Clear;
        }
        if record.bool("fc")? {
            lamp = Lamp::FC;
        }

        let instrument: Instrument = record.string_enum("instrument")?;

        let performance_data = Performance {
            common: ctx.create_common_p(record)?,
            instrument,
            difficulty: record.string_enum("difficulty")?,
            lamp,
            score: record.int("score").or_skip()?,
            notes_hit: record.int("hit_notes")?,
            notes_total: record.int("total_notes")?,
            max_streak: record.int("note_streak")?,
        };
        let match_data = Match {
            common: ctx.create_common_m(record, &[&performance_data])?,
            mode: Mode::Quickplay,
            game_version: None,
        };
        Ok((Box::new(match_data), vec![Box::new(performance_data)]))
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseSongRecordResult {
        Err(Continue(BadRecordError::NotImplemented)) // TODO
    }
}
