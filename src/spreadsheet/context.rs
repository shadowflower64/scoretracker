use calamine::Hyperlink;
use chrono_tz::Tz;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::data::library::database::{LibraryDatabase, LibraryEntry};
use crate::data::scoreboard::r#match::CommonMatchInfo;
use crate::data::scoreboard::performance::{CommonPerformanceInfo, PerformanceTrait};
use crate::data::scoreboard::player::{Player, PlayerDatabase};
use crate::spreadsheet::record::Record;
use crate::spreadsheet::{BadRecordError, ParseRecordResult, RecordErrorWithContext, SkipOrQuit};
use crate::util::uuid::UuidString;
use crate::util::youtube_id;

pub struct Context<'a> {
    pub player_database: &'a PlayerDatabase,
    pub library_database: &'a LibraryDatabase,
    pub proofs_to_insert: Vec<LibraryEntry>,
    pub tz: Tz,
    pub incomplete_match_records: Vec<RecordErrorWithContext>,
    pub incomplete_song_records: Vec<RecordErrorWithContext>,
    pub throwaway_match_record_count: u32,
    pub throwaway_song_record_count: u32,
}

impl Context<'_> {
    pub fn find_player_by_name(&self, name: &str) -> Result<&Player, BadRecordError> {
        self.player_database
            .find_player_by_name(name)
            .ok_or_else(|| BadRecordError::PlayerDoesNotExist { name: name.to_owned() })
    }

    pub fn find_proof_by_youtube_id(&self, youtube_id: &str) -> Result<&LibraryEntry, BadRecordError> {
        if let Some(proof) = self
            .proofs_to_insert
            .iter()
            .find(|x| x.youtube_id.as_ref().is_some_and(|id| id == youtube_id))
        {
            return Ok(proof);
        }

        if let Some(proof) = self.library_database.find_entry_by_youtube_id(youtube_id) {
            return Ok(proof);
        }

        Err(BadRecordError::ProofDoesNotExist {
            youtube_id: youtube_id.to_owned(),
        })
    }

    pub fn get_or_insert_proof_by_hyperlink(&mut self, hyperlink: &Hyperlink) -> Result<UuidString, BadRecordError> {
        let url = hyperlink.target.as_ref().expect("todo");
        let Some(youtube_id) = youtube_id(url) else {
            return Err(BadRecordError::InvalidYouTubeUrl { url: url.to_owned() });
        };
        if let Ok(proof) = self.find_proof_by_youtube_id(&youtube_id) {
            Ok(proof.uuid)
        } else {
            let proof = LibraryEntry {
                youtube_id: Some(youtube_id),
                ..Default::default()
            };
            let uuid = proof.uuid;
            self.proofs_to_insert.push(proof);
            Ok(uuid)
        }
    }

    fn create_proof(&mut self, record: &Record) -> Result<Vec<UuidString>, BadRecordError> {
        if let Some(string) = record.string_var("video")? {
            if string == ":(" || string == "-" {
                // `:(` => Proof got corrupted before it could be uploaded.
                // `-` => Proof never existed at all most likely.
                return Ok(Vec::new());
            }
        }
        if let Some(hyperlink) = record.hyperlink_var("video")? {
            if hyperlink.displayed_text == Some("YouTube".to_string()) {
                let proof_uuid = self.get_or_insert_proof_by_hyperlink(hyperlink)?;
                return Ok(vec![proof_uuid]);
            }
        }
        if record.is_empty("video")? {
            return Ok(Vec::new());
        }

        Err(BadRecordError::NotAHyperlink(
            "video".into(),
            Box::new(record.field_value("video")?.to_owned()),
        ))
    }

    pub fn create_common_p(&mut self, record: &Record) -> Result<CommonPerformanceInfo, BadRecordError> {
        let comment = match record.field_value("comment") {
            Ok(value) => Some(
                value
                    .as_string()
                    .ok_or(BadRecordError::NotAString("comment".into(), Box::new(value.to_owned())))?
                    .to_owned(),
            ),
            Err(_) => None,
        };
        Ok(CommonPerformanceInfo {
            uuid: Uuid::now_v7().into(),
            player_uuid: self.find_player_by_name(&record.string("player")?)?.uuid,
            proof: self.create_proof(record)?,
            comment,
            metadata: IndexMap::new(),
        })
    }

    pub fn create_common_m<P: PerformanceTrait>(&mut self, record: &Record, performances: &[&P]) -> ParseRecordResult<CommonMatchInfo> {
        Ok(CommonMatchInfo {
            uuid: Uuid::now_v7().into(),
            timestamp: record.timestamp("timestamp", self.tz).or_skip()?,
            song_id: record.string("song_id")?.to_owned(),
            performance_ids: performances.iter().map(|x| *x.uuid()).collect(),
            proof: Vec::new(),
            comment: None,
            metadata: IndexMap::new(),
        })
    }

    pub fn check_early_skip(&mut self, record: &Record) -> ParseRecordResult<()> {
        record.timestamp("timestamp", self.tz).or_skip()?;
        Ok(())
    }
}
