//! Data structures for DEEMO II.
//!
//! Progress status: All fields from the original spreadsheet are implemented.

use crate::data::game::Game;
use crate::data::scoreboard::r#match::{CommonMatchInfo, MatchTrait};
use crate::data::scoreboard::performance::{CommonPerformanceInfo, PerformanceTrait};
use crate::spreadsheet::ContinueOrQuit::Continue;
use crate::spreadsheet::context::Context;
use crate::spreadsheet::{BadRecordError, record::Record};
use crate::spreadsheet::{ParseMatchRecordResult, ParseSongRecordResult};
use crate::util::command_line::AskError;
use crate::util::percentage::Percentage;
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

#[typetag::serde(name = "deemo2")]
impl MatchTrait for Match {
    fn common(&self) -> &CommonMatchInfo {
        &self.common
    }
    fn sorting_key(&self) -> f64 {
        todo!()
    }
}

/// Difficulty that the performance was played on.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl TryFrom<&str> for Difficulty {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "easy" => Ok(Self::Easy),
            "normal" => Ok(Self::Normal),
            "hard" => Ok(Self::Hard),
            _ => Err("deemo2::Difficulty"),
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
}

impl TryFrom<&Record> for Lamp {
    type Error = BadRecordError;
    fn try_from(_record: &Record) -> Result<Self, Self::Error> {
        let lamp = Lamp::None;
        // if record.bool("c")? {
        //     lamp = Lamp::C;
        // }
        // if record.bool("fc")? {
        //     lamp = Lamp::FC;
        // }
        // if record.bool("pf")? {
        //     lamp = Lamp::PF;
        // }
        // TODO
        Ok(lamp)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Performance {
    #[serde(flatten)]
    pub common: CommonPerformanceInfo,

    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    /// Clear type.
    pub lamp: Lamp,

    /// Count of colored perfect judgements.
    pub combo: u32,

    /// Count of perfect judgements.
    pub charming: u32,

    /// Total count of judgements.
    pub total: u32,

    /// Accuracy.
    pub accuracy: Percentage,
}

#[typetag::serde(name = "cytus")]
impl PerformanceTrait for Performance {
    fn common(&self) -> &CommonPerformanceInfo {
        &self.common
    }
    fn sorting_key(&self) -> f64 {
        self.accuracy.0
    }
    fn ask_for_performance_edit(&mut self) -> Result<(), AskError> {
        todo!()
    }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct Deemo2;

#[typetag::serde(name = "deemo2")]
impl Game for Deemo2 {
    fn pretty_name(&self) -> &'static str {
        "DEEMO II"
    }
    fn url_shortname(&self) -> &'static str {
        "deemo2"
    }

    fn create_match_and_performance_from_spreadsheet_record(&self, record: &Record, ctx: &mut Context) -> ParseMatchRecordResult {
        let performance_data = Performance {
            common: ctx.create_common_p(record)?,
            difficulty: record.string_enum("difficulty")?,
            lamp: record.try_into()?,
            charming: record.int("charming")?,
            total: record.int("total")?,
            combo: record.int("combo")?,
            accuracy: Percentage::from_multiplier(record.f64("accuracy")?),
        };
        let match_data = Match {
            common: ctx.create_common_m(record, &[&performance_data])?,
            game_version: None,
        };
        Ok((Box::new(match_data), vec![Box::new(performance_data)]))
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseSongRecordResult {
        Err(Continue(BadRecordError::NotImplemented)) // TODO
    }

    game_impl!();
}

register_game!(Deemo2);
