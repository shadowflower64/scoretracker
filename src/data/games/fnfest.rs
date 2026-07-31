//! Data structures for Fortnite Festival.
//!
//! Progress status: All fields from the original spreadsheet are implemented.
//! Spreadsheet bug: `!artist_title` field in the spreadsheet - song ID is not present, it should be present in the input data. (TODO)

use crate::data::game::IncompleteOrCritical::Incomplete;
use crate::data::game::{Game, ImportMatchResult, ImportSongResult, SkipOrQuit, SpreadsheetContext};
use crate::data::scoreboard::r#match::MatchTrait;
use crate::data::scoreboard::performance::PerformanceTrait;
use crate::data::scoreboard::{r#match::CommonMatchInfo, performance::CommonPerformanceInfo};
use crate::spreadsheet::RecordError;
use crate::{spreadsheet::record::Record, util::command_line::AskError};
use serde::{Deserialize, Serialize};

/// Game mode.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    MainStage,
    BattleStage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Match {
    #[serde(flatten)]
    pub common: CommonMatchInfo,

    /// Game mode that this match was played on.
    pub mode: Mode,

    /// Leaderboard placement at the time of achieving the score.
    pub leaderboard_placement: Option<u32>,

    /// String of the game version that was played on for this match.
    /// None for unknown.
    pub game_version: Option<String>,
}

#[typetag::serde(name = "fnfest")]
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
    Lead,      // 5K gamepad gameplay
    Bass,      // 5K gamepad gameplay
    Drums,     // 5K gamepad gameplay
    Vocals,    // 5K gamepad gameplay
    ProLead,   // 5-fret guitar gameplay
    ProBass,   // 5-fret guitar gameplay
    ProDrums,  // 4-lane MIDI drums gameplay
    ProVocals, // Mic gameplay
    #[serde(rename = "pro_drums+cymbals")]
    ProDrumsWithCymbals, // 4-lane MIDI drums (4 drum pads + 3 cymbals) gameplay
}

impl TryFrom<&str> for Instrument {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "lead" => Ok(Self::Lead),
            "bass" => Ok(Self::Bass),
            "drums" => Ok(Self::Drums),
            "vocals" => Ok(Self::Vocals),
            "pro_lead" => Ok(Self::ProLead),
            "pro_bass" => Ok(Self::ProBass),
            "pro_drums" => Ok(Self::ProDrums),
            "pro_vocals" => Ok(Self::ProVocals),
            "pro_drums+cymbals" => Ok(Self::ProDrumsWithCymbals),
            _ => Err("fnfest::Instrument"),
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

impl TryFrom<&str> for Difficulty {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "easy" => Ok(Self::Easy),
            "medium" => Ok(Self::Medium),
            "hard" => Ok(Self::Hard),
            "expert" => Ok(Self::Expert),
            _ => Err("fnfest::Difficulty"),
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
    PFC,
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

    /// How many notes were hit perfectly.
    pub perfect: u64,

    /// How many notes were hit not perfectly.
    pub great: u64,

    /// How many notes were missed.
    pub miss: u64,

    /// How many overhits happened.
    pub strike: u64,

    /// The maximum streak achieved during the performance.
    pub max_streak: u64,
}

impl Performance {
    /// Custom formula calculating the X-Accuracy of the performance.
    ///
    /// Ouputs a number from 0.0 to 1.0.
    /// 0.0 -- all missed; 1.0 -- all perfect.
    pub fn x_acc(&self) -> f64 {
        todo!("implement x_acc formula from spreadsheet");
    }

    /// Letter grade based on the output of [`Self::x_acc`].
    pub fn grade(&self) -> String {
        todo!("implement grade formula from spreadsheet");
    }
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
pub struct FortniteFestival;

#[typetag::serde(name = "fnfest")]
impl Game for FortniteFestival {
    fn pretty_name(&self) -> &'static str {
        "Fortnite Festival"
    }
    fn url_shortname(&self) -> &'static str {
        "fnfest"
    }

    fn create_match_and_performance_from_spreadsheet_record(&self, record: &Record, ctx: &mut SpreadsheetContext) -> ImportMatchResult {
        // println!("{record}");
        ctx.check_early_skip(record)?;
        let mut lamp = Lamp::None;
        if record.bool("c")? {
            lamp = Lamp::Clear;
        }
        if record.bool("fc")? {
            lamp = Lamp::FC;
        }
        if record.bool("pfc")? {
            lamp = Lamp::PFC;
        }

        let instrument: Instrument = record.string_enum("instrument")?;

        let performance_data = Performance {
            common: ctx.create_common_p(record)?,
            instrument,
            difficulty: record.string_enum("difficulty")?,
            lamp,
            score: record.int("high_score").or_skip()?,
            perfect: record.int("perfect")?,
            great: record.int("great")?,
            miss: record.int("miss")?,
            strike: record.int("strike")?,
            max_streak: record.int("note_streak")?,
        };
        let match_data = Match {
            common: ctx.create_common_m(record, &[&performance_data])?,
            mode: Mode::MainStage,
            leaderboard_placement: record.int_opt("leaderboard_placement")?,
            game_version: None,
        };
        Ok((Box::new(match_data), vec![Box::new(performance_data)]))
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut SpreadsheetContext) -> ImportSongResult {
        Err(Incomplete(RecordError::NotImplemented))
    }
}
