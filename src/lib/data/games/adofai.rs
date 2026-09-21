//! Data structures for A Dance of Fire and Ice.

use crate::data::game::Game;
use crate::data::scoreboard::r#match::Match;
use crate::data::scoreboard::r#match::MatchDetails;
use crate::data::scoreboard::performance::Performance;
use crate::data::scoreboard::performance::PerformanceDetails;
use crate::spreadsheet::BadRecordError;
use crate::spreadsheet::ContinueOrQuit::Continue;
use crate::spreadsheet::ParseMatchRecordResult;
use crate::spreadsheet::ParseSongRecordResult;
use crate::spreadsheet::SkipOrQuit;
use crate::spreadsheet::context::Context;
use crate::spreadsheet::record::Record;
use crate::util::command_line::AskError;
use crate::{game_impl, register_game};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub type JudgementCount = u32;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ADOFAIMatchDetails {}

#[typetag::serde(name = "adofai")]
impl MatchDetails for ADOFAIMatchDetails {
    fn sorting_key(&self) -> f64 {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Lamp {
    None,
    Clear,
    #[serde(rename = "fc")]
    FC,
    #[serde(rename = "perfect_fc")]
    PerfectFC,
    #[serde(rename = "pure_perfect_fc")]
    PurePerfectFC,
    #[serde(rename = "strict_pure_perfect_fc")]
    StrictPurePerfectFC,
}

impl TryFrom<&Record> for Lamp {
    type Error = BadRecordError;
    fn try_from(record: &Record) -> Result<Self, Self::Error> {
        let mut lamp = Lamp::None;
        if record.bool("c")? {
            lamp = Lamp::Clear;
        }
        if record.bool("fc")? {
            lamp = Lamp::FC;
        }
        if record.bool("pf")? {
            lamp = Lamp::PerfectFC;
        }
        if record.bool("pf+")? {
            lamp = Lamp::PurePerfectFC;
        }
        if record.bool_opt("strictpf+")?.unwrap_or(false) {
            lamp = Lamp::StrictPurePerfectFC;
        }
        Ok(lamp)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ADOFAIPerformanceDetails {
    pub lamp: Lamp,
    pub misses: JudgementCount,
    pub overload: JudgementCount,
    pub too_early: JudgementCount,
    pub early: JudgementCount,
    pub late: JudgementCount,
    pub early_perfect: JudgementCount,
    pub late_perfect: JudgementCount,
    pub perfect: JudgementCount,
    pub checkpoints_used: u32,
}

impl ADOFAIPerformanceDetails {
    pub fn total_tiles(&self) -> JudgementCount {
        self.misses + self.early + self.late + self.early_perfect + self.late_perfect + self.perfect
    }

    pub fn accuracy(&self) -> f64 {
        let all_judgements = self.misses
            + self.overload
            + self.too_early
            + self.early
            + self.late
            + self.early_perfect
            + self.late_perfect
            + self.perfect
            + self.misses // TODO: misses and overloads are added twice, i think this is intentional??? This is how it is in the original sheet
            + self.overload;
        if all_judgements == 0 {
            0.0
        } else {
            let all_perfects = self.early_perfect + self.late_perfect + self.perfect;
            let base_percentage = (all_perfects as f64) / (all_judgements as f64);
            let bonus_percentage = (self.perfect as f64) * 0.0001f64;
            base_percentage + bonus_percentage
        }
    }

    pub fn x_accuracy(&self) -> f64 {
        let all_judgements =
            self.misses + self.overload + self.too_early + self.early + self.late + self.early_perfect + self.late_perfect + self.perfect;
        if all_judgements == 0 {
            0.0
        } else {
            let base_percentage = ((self.perfect as f64 * 1.0f64)
                + ((self.early_perfect + self.late_perfect) as f64 * 0.75f64)
                + ((self.early + self.late) as f64 * 0.4f64)
                + (self.too_early as f64 * 0.2f64))
                / all_judgements as f64;
            let checkpoint_factor = 0.9875f64.powi(self.checkpoints_used as i32);
            base_percentage * checkpoint_factor
        }
    }
}

#[typetag::serde(name = "adofai")]
impl PerformanceDetails for ADOFAIPerformanceDetails {
    fn sorting_key(&self) -> f64 {
        self.x_accuracy()
    }
    fn ask_for_performance_edit(&mut self) -> Result<(), AskError> {
        todo!()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ADOFAI;

#[typetag::serde(name = "adofai")]
impl Game for ADOFAI {
    fn pretty_name(&self) -> &'static str {
        "A Dance of Fire and Ice"
    }
    fn url_shortname(&self) -> &'static str {
        "adofai"
    }

    fn create_match_and_performance_from_spreadsheet_record(&self, record: &Record, _ctx: &mut Context) -> ParseMatchRecordResult {
        let perfect = record.int("perfect").or_skip()?;
        let match_details = ADOFAIMatchDetails {};
        let performance_details = ADOFAIPerformanceDetails {
            lamp: record.try_into()?,
            misses: record.int("misses")?,
            overload: record.int("overhits")?,
            too_early: record.int("too_early")?,
            early: record.int("early")?,
            late: record.int("late")?,
            early_perfect: record.int("early_perfect")?,
            late_perfect: record.int("late_perfect")?,
            perfect,
            checkpoints_used: record.int("checkpoints_used")?,
        };
        Ok((Box::new(match_details), vec![Box::new(performance_details)]))
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseSongRecordResult {
        // Err(Critical(RecordError::NotImplemented))
        Err(Continue(BadRecordError::NotImplemented)) // TODO
    }

    game_impl!();
}

register_game!(ADOFAI);
