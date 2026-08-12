//! Data structures for Hatsune Miku: Project DIVA Mega Mix+.

use crate::data::game::Game;
use crate::data::scoreboard::r#match::{CommonMatchInfo, MatchTrait};
use crate::data::scoreboard::performance::{CommonPerformanceInfo, PerformanceTrait};
use crate::spreadsheet::ContinueOrQuit::Continue;
use crate::spreadsheet::{BadRecordError, ParseSongRecordResult, context::Context, record::Record};
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

#[typetag::serde(name = "pjd_megamix_plus")]
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
    Extreme,
    ExtraExtreme,
}

impl TryFrom<&str> for Difficulty {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "easy" => Ok(Self::Easy),
            "normal" => Ok(Self::Normal),
            "hard" => Ok(Self::Hard),
            "extreme" => Ok(Self::Extreme),
            "extra_extreme" => Ok(Self::ExtraExtreme),
            _ => Err("pjd_megamix_plus::Difficulty"),
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
pub struct Performance {
    #[serde(flatten)]
    pub common: CommonPerformanceInfo,

    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    /// Clear type.
    pub lamp: Lamp,

    /// Count of cool judgements.
    pub cool: u32,

    /// Count of safe judgements.
    pub safe: u32,

    /// Count of bad judgements.
    pub bad: u32,

    /// Count of miss judgements.
    pub miss: u32,

    /// Count of wrong judgements.
    pub wrong: u32,

    /// Count of almost judgements.
    pub almost: u32,

    /// Score in range [0..1_000_000].
    pub score: u32,
}

#[typetag::serde(name = "pjd_megamix_plus")]
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
pub struct ProjectDIVAMegaMixPlus;

#[typetag::serde(name = "pjd_megamix_plus")]
impl Game for ProjectDIVAMegaMixPlus {
    fn pretty_name(&self) -> &'static str {
        "Hatsune Miku: Project DIVA Mega Mix+"
    }
    fn url_shortname(&self) -> &'static str {
        "pjd_megamix_plus"
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseSongRecordResult {
        Err(Continue(BadRecordError::NotImplemented)) // TODO
    }

    game_impl!();
}

register_game!(ProjectDIVAMegaMixPlus);
