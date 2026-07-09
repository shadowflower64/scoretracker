//! Data structures for A Dance of Fire and Ice
use std::collections::HashMap;

use crate::game::Game;
use crate::scoreboard::r#match::CommonMatchInfo;
use crate::scoreboard::r#match::MatchTrait;
use crate::scoreboard::performance::CommonPerformanceInfo;
use crate::scoreboard::performance::PerformanceTrait;
use crate::songdb::song::SongTrait;
use crate::spreadsheet::Record;
use crate::spreadsheet::SpreadsheetRecordImportError;
use crate::spreadsheet::find_player_uuid_by_name;
use crate::spreadsheet::get_or_insert_proof_by_youtube_url;
use crate::util::command_line::AskError;
use serde::{Deserialize, Serialize};
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
    fn score(&self) -> f64 {
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
    pub overhits: JudgementCount,
    pub too_early: JudgementCount,
    pub early: JudgementCount,
    pub late: JudgementCount,
    pub early_perfect: JudgementCount,
    pub late_perfect: JudgementCount,
    pub perfect: JudgementCount,
    pub checkpoints_used: u32,
}

#[typetag::serde(name = "adofai")]
impl PerformanceTrait for Performance {
    fn common(&self) -> &CommonPerformanceInfo {
        &self.common
    }
    fn score(&self) -> f64 {
        todo!()
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
                player_uuid: find_player_uuid_by_name(&record.string("player")?).expect("todo"),
                proof: vec![get_or_insert_proof_by_youtube_url(&record.hyperlink("youtube")?)],
                comment: record.string_opt("comment")?,
                metadata: HashMap::new(),
            },
            lamp,
            misses: record.int("misses")?,
            overhits: record.int("overhits")?,
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

    fn create_song_from_spreadsheet_record(&self, _record: &Record) -> Result<Box<dyn SongTrait>, SpreadsheetRecordImportError> {
        Err(SpreadsheetRecordImportError::NotImplemented)
    }
}
