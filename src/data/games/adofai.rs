//! Data structures for A Dance of Fire and Ice
use crate::data::game::Game;
use crate::data::game::SpreadsheetContext;
use crate::data::game::song::SongTrait;
use crate::data::scoreboard::r#match::CommonMatchInfo;
use crate::data::scoreboard::r#match::MatchTrait;
use crate::data::scoreboard::performance::CommonPerformanceInfo;
use crate::data::scoreboard::performance::PerformanceTrait;
use crate::spreadsheet::Record;
use crate::spreadsheet::SpreadsheetRecordImportError;
use crate::spreadsheet::get_or_insert_proof_by_youtube_url;
use crate::util::command_line::AskError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
pub type JudgementCount = u32;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Match {
    pub common: CommonMatchInfo,
}

#[typetag::serde(name = "adofai")]
impl MatchTrait for Match {
    fn common(&self) -> &CommonMatchInfo {
        &self.common
    }
    fn sorting_key(&self) -> f64 {
        todo!()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Lamp {
    None,
    Clear,
    FC,
    PerfectFC,
    PurePerfectFC,
    StrictPurePerfectFC,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Performance {
    pub common: CommonPerformanceInfo,
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

impl Performance {
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
impl PerformanceTrait for Performance {
    fn common(&self) -> &CommonPerformanceInfo {
        &self.common
    }
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

    fn create_match_and_performance_from_spreadsheet_record(
        &self,
        record: &Record,
        ctx: SpreadsheetContext,
    ) -> Result<(Box<dyn MatchTrait>, Vec<Box<dyn PerformanceTrait>>), SpreadsheetRecordImportError> {
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
        if record.bool_or("strictpf+", false)? {
            lamp = Lamp::StrictPurePerfectFC;
        }
        let performance_data = Performance {
            common: CommonPerformanceInfo {
                uuid: Uuid::now_v7().into(),
                player_uuid: ctx.find_player_by_name(&record.string("player")?)?.uuid,
                proof: vec![get_or_insert_proof_by_youtube_url(
                    &record.hyperlink("video")?.target.expect("todo"),
                )],
                comment: record.string_opt("comment")?,
                metadata: HashMap::new(),
            },
            lamp,
            misses: record.int("misses")?,
            overload: record.int("overhits")?,
            too_early: record.int("too_early")?,
            early: record.int("early")?,
            late: record.int("late")?,
            early_perfect: record.int("early_perfect")?,
            late_perfect: record.int("late_perfect")?,
            perfect: record.int("perfect")?,
            checkpoints_used: record.int("checkpoints_used")?,
        };
        let match_data = Match {
            common: CommonMatchInfo {
                uuid: Uuid::now_v7().into(),
                timestamp: record.timestamp("timestamp")?,
                song_id: record.string("song_id")?,
                performance_ids: vec![*performance_data.uuid()],
                proof: Vec::new(),
                comment: record.string_opt("comment")?,
                metadata: HashMap::new(),
            },
        };
        Ok((Box::new(match_data), vec![Box::new(performance_data)]))
    }

    fn create_song_from_spreadsheet_record(
        &self,
        _record: &Record,
        _ctx: SpreadsheetContext,
    ) -> Result<Box<dyn SongTrait>, SpreadsheetRecordImportError> {
        Err(SpreadsheetRecordImportError::NotImplemented)
    }
}
