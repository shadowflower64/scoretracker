//! Data structures for Beatstar.

use crate::data::scoreboard::r#match::{Match, MatchDetails};
use crate::data::scoreboard::performance::{Performance, PerformanceDetails};
use crate::game_impl;
use crate::spreadsheet::BadRecordError;
use crate::spreadsheet::ContinueOrQuit::Continue;
use crate::spreadsheet::ParseChartsetRecordResult;
use crate::spreadsheet::context::Context;
use crate::spreadsheet::record::Record;
use crate::util::command_line::AskError;
use crate::{data::game::Game, register_game};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct BeatstarMatchDetails {
    /// String of the game version that was played on for this match.
    /// None for unknown.
    pub game_version: Option<String>,
}

#[typetag::serde(name = "beatstar")]
impl MatchDetails for BeatstarMatchDetails {
    fn sorting_key(&self) -> f64 {
        todo!()
    }
}

/// Difficulty that the performance was played on.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Normal,
    Deluxe,
}

impl TryFrom<&str> for Difficulty {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "normal" => Ok(Self::Normal),
            "deluxe" => Ok(Self::Deluxe),
            _ => Err("beatstar::Difficulty"),
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
pub struct BeatstarPerformanceDetails {
    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    /// Clear type.
    pub lamp: Lamp,

    /// Count of perfect judgements.
    pub perfect: u32,

    /// Count of good judgements.
    pub good: u32,

    /// Count of miss judgements.
    pub miss: u32,

    /// Score in range [0..1_000_000].
    pub score: u32,
}

#[typetag::serde(name = "beatstar")]
impl PerformanceDetails for BeatstarPerformanceDetails {
    fn sorting_key(&self) -> f64 {
        self.score as f64
    }
    fn ask_for_performance_edit(&mut self) -> Result<(), AskError> {
        todo!()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Beatstar;

#[typetag::serde(name = "beatstar")]
impl Game for Beatstar {
    fn pretty_name(&self) -> &'static str {
        "Beatstar"
    }
    fn url_shortname(&self) -> &'static str {
        "beatstar"
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseChartsetRecordResult {
        Err(Continue(BadRecordError::NotImplemented)) // TODO
    }

    game_impl!();
}

register_game!(Beatstar);
