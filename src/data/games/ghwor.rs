//! Data structures for Guitar Hero: Warriors of Rock.
//!
//! Progress status: All fields from the original spreadsheet are implemented.

use crate::data::game::IncompleteOrCritical::{self, Incomplete};
use crate::data::game::{Game, ImportMatchResult, ImportSongResult, SkipOrQuit, SpreadsheetContext};
use crate::data::scoreboard::r#match::MatchTrait;
use crate::data::scoreboard::performance::PerformanceTrait;
use crate::data::scoreboard::{r#match::CommonMatchInfo, performance::CommonPerformanceInfo};
use crate::spreadsheet::RecordError;
use crate::util::percentage::Percentage;
use crate::{spreadsheet::record::Record, util::command_line::AskError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Game mode.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Quest,
    Quickplay,
    PowerChallenge,
    Party,
    Practice,
}

#[derive(Error, Debug)]
#[error("not a mode: {0}")]
pub struct NotAMode(String);

impl From<NotAMode> for RecordError {
    fn from(value: NotAMode) -> Self {
        Self::Custom(Box::new(value))
    }
}

impl TryFrom<&str> for Mode {
    type Error = NotAMode;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "quest" => Ok(Self::Quest),
            "quickplay" => Ok(Self::Quickplay),
            "power_challenge" => Ok(Self::PowerChallenge),
            "party" => Ok(Self::Party),
            "practice" => Ok(Self::Practice),
            a => Err(NotAMode(a.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Match {
    #[serde(flatten)]
    pub common: CommonMatchInfo,

    /// Game mode that this match was played on.
    pub mode: Mode,

    /// Power Stars earned (present if playing in Quest mode or in Power Challenge).
    pub power_stars: Option<u8>,

    /// String of the game version that was played on for this match.
    /// None for unknown.
    pub game_version: Option<String>,
}

#[typetag::serde(name = "ghwor")]
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
    Vocals,
}

impl Instrument {
    pub fn make_note_stats(&self, record: &Record) -> Result<NoteStats, IncompleteOrCritical<RecordError>> {
        match self {
            Self::Guitar | Self::Bass | Self::Drums => Ok(NoteStats::Normal {
                notes_hit: record.int("hit_notes")?,
                notes_total: record.int("total_notes")?,
            }),
            Self::Vocals => {
                let hit: f64 = record.float("hit_notes")?;
                let total: f64 = record.float("total_notes")?;
                if total == 1.0f64 && hit != 1.0f64 {
                    // Some spreadsheet records just use a percentage value in place of the actual phrase count.
                    // This is because the stats screen doesn't actually show the amount of phrases hit correctly, just the percentage, because of different judgements.
                    Ok(NoteStats::VocalsAcc {
                        phrases_hit_percentage: Percentage::from_multiplier(hit / total),
                    })
                } else {
                    Ok(NoteStats::VocalsPhrases {
                        phrases_hit: record.int("hit_notes")?,
                        phrases_total: record.int("total_notes")?,
                    })
                    // unimplemented!("total was not 1.0, don't know how to calculate vocal phrase accuracy (total = {total})")
                }
            }
        }
    }
}

#[derive(Error, Debug)]
#[error("not an instrument: {0}")]
pub struct NotAnInstrument(String);

impl From<NotAnInstrument> for RecordError {
    fn from(value: NotAnInstrument) -> Self {
        Self::Custom(Box::new(value))
    }
}

impl TryFrom<&str> for Instrument {
    type Error = NotAnInstrument;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "guitar" => Ok(Self::Guitar),
            "bass" => Ok(Self::Bass),
            "drums" => Ok(Self::Drums),
            "vocals" => Ok(Self::Vocals),
            a => Err(NotAnInstrument(a.to_owned())),
        }
    }
}

/// Difficulty that the performance was played on.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Beginner,
    Easy,
    Medium,
    Hard,
    Expert,
    #[serde(rename = "expert+")]
    ExpertPlus,
}

#[derive(Error, Debug)]
#[error("not a difficulty: {0}")]
pub struct NotADifficulty(String);

impl From<NotADifficulty> for RecordError {
    fn from(value: NotADifficulty) -> Self {
        Self::Custom(Box::new(value))
    }
}

impl TryFrom<&str> for Difficulty {
    type Error = NotADifficulty;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "beginner" => Ok(Self::Beginner),
            "easy" => Ok(Self::Easy),
            "medium" => Ok(Self::Medium),
            "hard" => Ok(Self::Hard),
            "expert" => Ok(Self::Expert),
            "expert+" => Ok(Self::Expert),
            a => Err(NotADifficulty(a.to_owned())),
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
pub enum NoteStats {
    Normal {
        /// How many notes were hit successfully.
        notes_hit: u64,

        /// How many notes were in the chart (TODO: should be const across different scores of the chart)
        notes_total: u64,
    },
    VocalsAcc {
        /// Phrase accuracy.
        phrases_hit_percentage: Percentage,
    },
    VocalsPhrases {
        /// How many phrases were hit successfully.
        phrases_hit: u64,

        /// How many phrases were in the chart (TODO: should be const across different scores of the chart)
        phrases_total: u64,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Performance {
    #[serde(flatten)]
    pub common: CommonPerformanceInfo,

    ///// Who played what and when. /////
    /// Played instrument.
    pub instrument: Instrument,

    /// Difficulty level of the chart.
    pub difficulty: Difficulty,

    ///// Score and stats. /////
    /// Clear type.
    pub lamp: Lamp,

    /// Amount of points at the end of the performance.
    pub score: u64,

    /// Amount of hit notes or vocal phrases
    pub note_stats: NoteStats,

    /// The maximum streak achieved during the performance.
    pub max_streak: u64,
}

#[typetag::serde(name = "ghwor")]
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
pub struct GuitarHeroWarriorsOfRock;

#[typetag::serde(name = "ghwor")]
impl Game for GuitarHeroWarriorsOfRock {
    fn pretty_name(&self) -> &'static str {
        "Guitar Hero: Warriors of Rock"
    }
    fn url_shortname(&self) -> &'static str {
        "ghwor"
    }

    fn create_match_and_performance_from_spreadsheet_record(&self, record: &Record, ctx: &mut SpreadsheetContext) -> ImportMatchResult {
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
            note_stats: instrument.make_note_stats(record)?,
            max_streak: record.int("note_streak")?,
        };
        let match_data = Match {
            common: ctx.create_common_m(record, &[&performance_data])?,
            mode: record.string_enum("mode")?,
            power_stars: record.int_opt("power_stars")?,
            game_version: None,
        };
        Ok((Box::new(match_data), vec![Box::new(performance_data)]))
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut SpreadsheetContext) -> ImportSongResult {
        Err(Incomplete(RecordError::NotImplemented))
    }
}
