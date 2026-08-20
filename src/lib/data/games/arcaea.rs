//! Data structures for Arcaea.
//!
//! Progress status: All fields from the original spreadsheet are implemented.

use crate::data::game::Game;
use crate::data::scoreboard::r#match::{CommonMatchInfo, MatchTrait};
use crate::data::scoreboard::performance::{CommonPerformanceInfo, PerformanceTrait};
use crate::spreadsheet::BadRecordError;
use crate::spreadsheet::ContinueOrQuit::Continue;
use crate::spreadsheet::context::Context;
use crate::spreadsheet::record::Record;
use crate::spreadsheet::{ParseMatchRecordResult, ParseSongRecordResult};
use crate::util::command_line::AskError;
use crate::{game_impl, register_game};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Match {
    #[serde(flatten)]
    pub common: CommonMatchInfo,

    /// String of the game version that was played on for this match.
    /// None for unknown.
    pub game_version: Option<String>,
}

#[typetag::serde(name = "arcaea")]
impl MatchTrait for Match {
    fn common(&self) -> &CommonMatchInfo {
        &self.common
    }
    fn sorting_key(&self) -> f64 {
        unimplemented!()
    }
}

/// Game mode.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Normal,
    World,
    UnknownSingle,
}

/// Difficulty that the performance was played on.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Past,
    Present,
    Future,
    Beyond,
}

impl TryFrom<&str> for Difficulty {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "past" => Ok(Self::Past),
            "present" => Ok(Self::Present),
            "future" => Ok(Self::Future),
            "beyond" => Ok(Self::Beyond),
            _ => Err("arcaea::Difficulty"),
        }
    }
}

/// Clear type.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Lamp {
    None,
    C,
    FC,
    PF,
    #[serde(rename = "pf_plus")]
    PFPlus,
}

impl TryFrom<&Record> for Lamp {
    type Error = BadRecordError;
    fn try_from(record: &Record) -> Result<Self, Self::Error> {
        let mut lamp = Lamp::None;
        if record.bool("c")? {
            lamp = Lamp::C;
        }
        if record.bool("fc")? {
            lamp = Lamp::FC;
        }
        if record.bool("pf")? {
            lamp = Lamp::PF;
        }
        if record.bool("pf+")? {
            lamp = Lamp::PFPlus;
        }
        Ok(lamp)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Performance {
    #[serde(flatten)]
    pub common: CommonPerformanceInfo,

    /// Mode that this performance was played on.
    pub mode: Mode,

    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    /// Clear type.
    pub lamp: Lamp,

    /// Count of shiny pure judgements.
    pub shiny_pure: u32,

    /// Count of pure judgements.
    pub pure: u32,

    /// Count of far judgements.
    pub far: u32,

    /// Count of lost judgements.
    pub lost: u32,

    /// Score in range [0..10_000_000+note_count]. Only present for Normal mode.
    pub score: u32,
}

impl Performance {}

#[typetag::serde(name = "arcaea")]
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
pub struct Arcaea;

#[typetag::serde(name = "arcaea")]
impl Game for Arcaea {
    fn pretty_name(&self) -> &'static str {
        "Arcaea"
    }
    fn url_shortname(&self) -> &'static str {
        "arcaea"
    }

    fn create_match_and_performance_from_spreadsheet_record(&self, record: &Record, ctx: &mut Context) -> ParseMatchRecordResult {
        let match_data = Match {
            common: ctx.create_common_m(record)?,
            game_version: None,
        };
        let performance_data = Performance {
            common: ctx.create_common_p(record, match_data.uuid())?,
            mode: Mode::UnknownSingle,
            difficulty: record.string_enum("difficulty")?,
            lamp: record.try_into()?,
            shiny_pure: record.int("shiny_pure")?,
            pure: record.int("pure")?,
            far: record.int("far")?,
            lost: record.int("lost")?,
            score: record.int("score")?,
        };
        Ok((Box::new(match_data), vec![Box::new(performance_data)]))
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseSongRecordResult {
        Err(Continue(BadRecordError::NotImplemented)) // TODO
    }

    game_impl!();
}

register_game!(Arcaea);
