//! Data structures for Phigros.

use crate::data::scoreboard::r#match::{Match, MatchDetails};
use crate::data::scoreboard::performance::{Performance, PerformanceDetails};
use crate::spreadsheet::ContinueOrQuit::Continue;
use crate::spreadsheet::record::Record;
use crate::spreadsheet::{BadRecordError, ParseChartsetRecordResult};
use crate::util::command_line::AskError;
use crate::{data::game::Game, spreadsheet::context::Context};
use crate::{game_impl, register_game};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PhigrosMatchDetails {
    /// String of the game version that was played on for this match.
    /// None for unknown.
    pub game_version: Option<String>,
}

#[typetag::serde(name = "phigros")]
impl MatchDetails for PhigrosMatchDetails {
    fn sorting_key(&self) -> f64 {
        todo!()
    }
}

/// Game mode.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Normal,
    Challenge,
}

/// Difficulty that the performance was played on.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Ez,
    Hd,
    In,
    At,
}

impl TryFrom<&str> for Difficulty {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ez" => Ok(Self::Ez),
            "hd" => Ok(Self::Hd),
            "in" => Ok(Self::In),
            "at" => Ok(Self::At),
            _ => Err("phigros::Difficulty"),
        }
    }
}

/// Clear type.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Lamp {
    None,
    Clear,
    FC,
    PF,
}

impl TryFrom<&Record> for Lamp {
    type Error = BadRecordError;
    fn try_from(record: &Record) -> Result<Self, Self::Error> {
        let mut lamp = Lamp::None;
        if record.bool("clear")? {
            lamp = Lamp::Clear;
        }
        if record.bool("fc")? {
            lamp = Lamp::FC;
        }
        if record.bool("pf")? {
            lamp = Lamp::PF;
        }
        Ok(lamp)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PhigrosPerformanceDetails {
    /// Mode that this performance was played on.
    pub mode: Mode,

    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    /// Clear type.
    pub lamp: Lamp,

    /// Count of perfect judgements.
    pub perfect: u32,

    /// Count of good judgements.
    pub good: u32,

    /// Count of bad judgements.
    pub bad: u32,

    /// Count of miss judgements.
    pub miss: u32,

    /// Score in range [0..1_000_000].
    pub score: u32,
}

#[typetag::serde(name = "phigros")]
impl PerformanceDetails for PhigrosPerformanceDetails {
    fn sorting_key(&self) -> f64 {
        self.score as f64
    }
    fn ask_for_performance_edit(&mut self) -> Result<(), AskError> {
        todo!()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Phigros;

#[typetag::serde(name = "phigros")]
impl Game for Phigros {
    fn pretty_name(&self) -> &'static str {
        "Phigros"
    }
    fn url_shortname(&self) -> &'static str {
        "phigros"
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseChartsetRecordResult {
        Err(Continue(BadRecordError::NotImplemented)) // TODO
    }

    game_impl!();
}

register_game!(Phigros); // TODO
