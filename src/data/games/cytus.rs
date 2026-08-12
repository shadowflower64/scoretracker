//! Data structures for Cytus.
//!
//! Progress status: All fields from the original spreadsheet are implemented.

use crate::data::game::Game;
use crate::data::scoreboard::r#match::{CommonMatchInfo, MatchTrait};
use crate::data::scoreboard::performance::{CommonPerformanceInfo, PerformanceTrait};
use crate::spreadsheet::ContinueOrQuit::Continue;
use crate::spreadsheet::context::Context;
use crate::spreadsheet::{BadRecordError, record::Record};
use crate::spreadsheet::{ParseMatchRecordResult, ParseSongRecordResult, SkipOrQuit};
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

#[typetag::serde(name = "cytus")]
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
    Hard,
}

impl TryFrom<&str> for Difficulty {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "easy" => Ok(Self::Easy),
            "hard" => Ok(Self::Hard),
            _ => Err("cytus::Difficulty"),
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

    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    /// Clear type.
    pub lamp: Lamp,

    /// Count of colored perfect judgements.
    pub color_perfect: u32,

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

impl Performance {
    pub fn tp(&self) -> Percentage {
        todo!("implement TP formula from spreadsheet")
    }
}

#[typetag::serde(name = "cytus")]
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
pub struct Cytus;

#[typetag::serde(name = "cytus")]
impl Game for Cytus {
    fn pretty_name(&self) -> &'static str {
        "Cytus"
    }
    fn url_shortname(&self) -> &'static str {
        "cytus"
    }

    fn create_match_and_performance_from_spreadsheet_record(&self, record: &Record, ctx: &mut Context) -> ParseMatchRecordResult {
        let score = record.int("score").or_skip()?;
        let match_data = Match {
            common: ctx.create_common_m(record)?,
            game_version: None,
        };
        let performance_data = Performance {
            common: ctx.create_common_p(record, match_data.uuid())?,
            difficulty: record.string_enum("difficulty")?,
            lamp: record.try_into()?,
            color_perfect: record.int("color_perfect")?,
            perfect: record.int("perfect")?,
            good: record.int("good")?,
            bad: record.int("bad")?,
            miss: record.int("miss")?,
            score,
        };
        Ok((Box::new(match_data), vec![Box::new(performance_data)]))
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseSongRecordResult {
        Err(Continue(BadRecordError::NotImplemented)) // TODO
    }

    game_impl!();
}

register_game!(Cytus);
