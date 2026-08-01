//! Data structures for DEEMO II.
//!
//! Progress status: All fields from the original spreadsheet are implemented.

use crate::data::game::IncompleteOrCritical::Incomplete;
use crate::data::game::{Game, ImportMatchResult, ImportSongResult, SpreadsheetContext};
use crate::data::scoreboard::r#match::{CommonMatchInfo, MatchTrait};
use crate::data::scoreboard::performance::{CommonPerformanceInfo, PerformanceTrait};
use crate::spreadsheet::{RecordError, record::Record};
use crate::util::command_line::AskError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Match {
    #[serde(flatten)]
    pub common: CommonMatchInfo,

    /// String of the game version that was played on for this match.
    /// None for unknown.
    pub game_version: Option<String>,
}

#[typetag::serde(name = "vs")]
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
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Opening,
    Middle,
    Finale,
    Encore,
}

impl TryFrom<&str> for Difficulty {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "opening" => Ok(Self::Opening),
            "middle" => Ok(Self::Middle),
            "finale" => Ok(Self::Finale),
            "encore" => Ok(Self::Encore),
            _ => Err("vs::Difficulty"),
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
    AC,
    VS,
}

impl TryFrom<&Record> for Lamp {
    type Error = RecordError;
    fn try_from(record: &Record) -> Result<Self, Self::Error> {
        let mut lamp = Lamp::None;
        if record.bool("c")? {
            lamp = Lamp::C;
        }
        if record.bool("fc")? {
            lamp = Lamp::FC;
        }
        if record.bool("pf")? {
            lamp = Lamp::AC;
        }
        if record.bool("pf+")? {
            lamp = Lamp::VS;
        }
        Ok(lamp)
    }
}

/// Rank.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Rank {
    F,
    E,
    D,
    C,
    B,
    A,
    AA,
    S,
    SPlus,
    SS,
    SSPlus,
    V,
    VPlus,
    VS,
}

impl TryFrom<&str> for Rank {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "F" => Ok(Self::F),
            "E" => Ok(Self::E),
            "D" => Ok(Self::D),
            "C" => Ok(Self::C),
            "B" => Ok(Self::B),
            "A" => Ok(Self::A),
            "AA" => Ok(Self::AA),
            "S" => Ok(Self::S),
            "S+" => Ok(Self::SPlus),
            "SS" => Ok(Self::SS),
            "SS+" => Ok(Self::SSPlus),
            "V" => Ok(Self::V),
            "V+" => Ok(Self::VPlus),
            "VS" => Ok(Self::VS),
            _ => Err("vs::Rank"),
        }
    }
}

impl Rank {
    pub fn from_score(score: u32, failed: bool) -> Option<Self> {
        if failed {
            return Some(Self::F);
        }
        match score {
            1_010_000 => Some(Self::VS),
            1_009_000..1_010_000 => Some(Self::VPlus),
            1_008_000..1_009_000 => Some(Self::V),
            1_004_000..1_008_000 => Some(Self::SSPlus),
            1_000_000..1_004_000 => Some(Self::SS),
            990_000..1_000_000 => Some(Self::SPlus),
            980_000..990_000 => Some(Self::S),
            950_000..980_000 => Some(Self::AA),
            900_000..950_000 => Some(Self::A),
            850_000..900_000 => Some(Self::B),
            800_000..850_000 => Some(Self::C),
            600_000..800_000 => Some(Self::D),
            0..600_000 => Some(Self::E),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Performance {
    #[serde(flatten)]
    pub common: CommonPerformanceInfo,

    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    /// Clear type.
    pub lamp: Lamp,

    /// Rank achieved during the performance.
    pub rank: Rank,

    /// Count of good-early judgements.
    pub good_early: u32,

    /// Count of great-early judgements.
    pub great_early: u32,

    /// Count of critical-early judgements.
    pub crit_early: u32,

    /// Count of accurate-critical judgements.
    pub accurate_crit: u32,

    /// Count of critical-late judgements.
    pub crit_late: u32,

    /// Count of great-late judgements.
    pub great_late: u32,

    /// Count of good-late judgements.
    pub good_late: u32,

    /// Count of failed judgements.
    pub failed: u32,

    /// Total score in the range [0..10_010_000].
    pub score: u32,

    /// Best combo achieved during a play.
    pub combo: u32,

    /// Play rating.
    pub play_rate: f64,

    /// EX score.
    pub ex: u32,
}

#[typetag::serde(name = "vs")]
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
pub struct VividStasis;

#[typetag::serde(name = "vs")]
impl Game for VividStasis {
    fn pretty_name(&self) -> &'static str {
        "vivid/stasis"
    }
    fn url_shortname(&self) -> &'static str {
        "vs"
    }

    fn create_match_and_performance_from_spreadsheet_record(&self, record: &Record, ctx: &mut SpreadsheetContext) -> ImportMatchResult {
        // println!("{record}");
        let performance_data = Performance {
            common: ctx.create_common_p(record)?,
            difficulty: record.string_enum("difficulty")?,
            lamp: record.try_into()?,
            combo: record.int("combo")?,
            rank: record.string_enum("rank")?,
            good_early: record.int("good_early")?,
            great_early: record.int("great_early")?,
            crit_early: record.int("crit_early")?,
            accurate_crit: record.int("accurate_crit")?,
            crit_late: record.int("crit_late")?,
            great_late: record.int("great_late")?,
            good_late: record.int("good_late")?,
            failed: record.int("failed")?,
            score: record.int("score")?,
            play_rate: record.float("play_rate")?, // TODO: this can be calculated automatically
            ex: record.int("ex")?,
        };
        let match_data = Match {
            common: ctx.create_common_m(record, &[&performance_data])?,
            game_version: None,
        };
        Ok((Box::new(match_data), vec![Box::new(performance_data)]))
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut SpreadsheetContext) -> ImportSongResult {
        Err(Incomplete(RecordError::NotImplemented)) // TODO
    }
}
