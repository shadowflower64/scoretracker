//! Data structures for Rizline.
//!
//! Progress status: All fields from the original spreadsheet are implemented.
//! Spreadsheet bug: Rizline sheet has 2 max_combo fields (TODO)

use crate::data::game::IncompleteOrCritical::Incomplete;
use crate::data::game::{Game, ImportMatchResult, ImportSongResult, SkipOrQuit, SpreadsheetContext};
use crate::data::scoreboard::r#match::{CommonMatchInfo, MatchTrait};
use crate::data::scoreboard::performance::{CommonPerformanceInfo, PerformanceTrait};
use crate::spreadsheet::{RecordError, record::Record};
use crate::util::command_line::AskError;
use crate::util::percentage::Percentage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Match {
    #[serde(flatten)]
    pub common: CommonMatchInfo,

    /// String of the game version that was played on for this match.
    /// None for unknown.
    pub game_version: Option<String>,
}

#[typetag::serde(name = "rizline")]
impl MatchTrait for Match {
    fn common(&self) -> &CommonMatchInfo {
        &self.common
    }
    fn sorting_key(&self) -> f64 {
        todo!()
    }
}

/// Difficulty that the performance was played on.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    EZ,
    HD,
    IN,
    AT,
}

impl TryFrom<&str> for Difficulty {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ez" => Ok(Self::EZ),
            "hd" => Ok(Self::HD),
            "in" => Ok(Self::IN),
            "at" => Ok(Self::AT),
            _ => Err("rizline::Difficulty"),
        }
    }
}

/// Clear type.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Lamp {
    None,
    C,
    FC,
    PFC,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Performance {
    #[serde(flatten)]
    pub common: CommonPerformanceInfo,

    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    /// Clear type.
    pub lamp: Lamp,

    /// How many big dots (grade dots) were achieved during the performance.
    pub stars: u8,

    /// How many hit points were achieved during the performance.
    pub hits: u64,

    /// How many hit points were possible to achieve.
    /// TODO: this should be a chart property.
    pub max_hits: u64,

    /// The biggest combo achieved during the performance.
    pub combo: u64,

    /// The maximum combo possible in this chart.
    /// TODO: this should be a chart property.
    pub max_combo: u64,

    /// Amount of score at the end of the performance.
    pub score: u64,

    /// How many notes were in the chart.
    /// TODO: this should be a chart property.
    pub max_score: u64,

    /// Clear rate percentage (from 0% to 120%).
    pub clear_rate: Percentage,
}

#[typetag::serde(name = "rizline")]
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
pub struct Rizline;

#[typetag::serde(name = "rizline")]
impl Game for Rizline {
    fn pretty_name(&self) -> &'static str {
        "Rizline"
    }
    fn url_shortname(&self) -> &'static str {
        "rizline"
    }

    fn create_match_and_performance_from_spreadsheet_record(&self, record: &Record, ctx: &mut SpreadsheetContext) -> ImportMatchResult {
        // println!("{record}");
        let mut lamp = Lamp::None;
        if record.bool("c")? {
            lamp = Lamp::C;
        }
        if record.bool("fc")? {
            lamp = Lamp::FC;
        }
        if record.bool("pfc")? {
            lamp = Lamp::PFC;
        }
        let score = record.int("score").or_skip()?;
        let performance_data = Performance {
            common: ctx.create_common_p(record)?,
            difficulty: record.string_enum("difficulty")?,
            lamp,
            stars: record.int("stars")?,
            hits: record.int("hits")?,
            max_hits: record.int("max_hits")?,
            combo: record.int("combo")?,
            max_combo: record.int("max_combo")?,
            score,
            max_score: record.int("max_score")?,
            clear_rate: Percentage::from_multiplier(record.float::<f64, _>("clear_rate")?),
        };
        let match_data = Match {
            common: ctx.create_common_m(record, &[&performance_data])?,
            game_version: None,
        };
        Ok((Box::new(match_data), vec![Box::new(performance_data)]))
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut SpreadsheetContext) -> ImportSongResult {
        Err(Incomplete(RecordError::NotImplemented))
    }
}
