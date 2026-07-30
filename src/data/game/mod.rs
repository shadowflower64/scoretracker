pub mod song;

use crate::data::game::IncompleteOrCritical::Critical;
use crate::data::game::song::AnySong;
use crate::data::library::database::{LibraryDatabase, LibraryEntry};
use crate::data::scoreboard::r#match::{AnyMatch, CommonMatchInfo};
use crate::data::scoreboard::performance::{AnyPerformance, CommonPerformanceInfo, PerformanceTrait};
use crate::data::scoreboard::player::{Player, PlayerDatabase};
use crate::spreadsheet::{FieldValue, Record, SpreadsheetRecordImportError};
use crate::util::command_line::AskError;
use crate::util::uuid::UuidString;
use crate::util::youtube_id;
use calamine::Hyperlink;
use chrono_tz::Tz;
use indexmap::IndexMap;
use serde::Serialize;
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Clone)]
pub struct SpreadsheetContext<'a> {
    pub player_database: &'a PlayerDatabase,
    pub library_database: &'a LibraryDatabase,
    pub proofs_to_insert: Vec<LibraryEntry>,
    pub tz: Tz,
}

impl SpreadsheetContext<'_> {
    pub fn find_player_by_name(&self, name: &str) -> Result<&Player, SpreadsheetRecordImportError> {
        self.player_database
            .find_player_by_name(name)
            .ok_or_else(|| SpreadsheetRecordImportError::PlayerDoesNotExist { name: name.to_owned() })
    }

    pub fn find_proof_by_youtube_id(&self, youtube_id: &str) -> Result<&LibraryEntry, SpreadsheetRecordImportError> {
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

        Err(SpreadsheetRecordImportError::ProofDoesNotExist {
            youtube_id: youtube_id.to_owned(),
        })
    }

    pub fn get_or_insert_proof_by_hyperlink(&mut self, hyperlink: &Hyperlink) -> Result<UuidString, SpreadsheetRecordImportError> {
        let url = hyperlink.target.as_ref().expect("todo");
        let Some(youtube_id) = youtube_id(url) else {
            return Err(SpreadsheetRecordImportError::InvalidYouTubeUrl { url: url.to_owned() });
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

    pub fn create_common_p(&mut self, record: &Record) -> Result<CommonPerformanceInfo, SpreadsheetRecordImportError> {
        let proof = match record.field_opt("video") {
            Some(FieldValue::String(a)) if a == ":(" => {
                // Proof got corrupted before it could be uploaded :(
                Vec::new()
            }
            Some(FieldValue::Hyperlink(hyperlink)) => {
                let proof_uuid = self.get_or_insert_proof_by_hyperlink(hyperlink)?;
                vec![proof_uuid]
            }
            None => {
                // Cell is empty
                Vec::new()
            }
            Some(value) => {
                // Invalid cell contents
                return Err(SpreadsheetRecordImportError::NotAHyperlink(
                    "video".into(),
                    Box::new(value.to_owned()),
                ));
            }
        };

        Ok(CommonPerformanceInfo {
            uuid: Uuid::now_v7().into(),
            player_uuid: self.find_player_by_name(&record.string("player")?)?.uuid,
            proof,
            comment: record.string_opt("comment")?,
            metadata: IndexMap::new(),
        })
    }

    pub fn create_common_m<P: PerformanceTrait>(&mut self, record: &Record, performances: &[&P]) -> RecordImportResult<CommonMatchInfo> {
        Ok(CommonMatchInfo {
            uuid: Uuid::now_v7().into(),
            timestamp: record.timestamp("timestamp", self.tz).or_skip()?,
            song_id: record.string("song_id")?,
            performance_ids: performances.iter().map(|x| *x.uuid()).collect(),
            proof: Vec::new(),
            comment: record.string_opt("comment")?,
            metadata: IndexMap::new(),
        })
    }
}

pub enum IncompleteOrCritical<E> {
    Incomplete(E),
    Critical(E),
}
pub type RecordImportResult<T> = Result<T, IncompleteOrCritical<SpreadsheetRecordImportError>>;

pub trait SkipOrQuit<T> {
    fn or_skip(self) -> RecordImportResult<T>;
    fn or_quit(self) -> RecordImportResult<T>;
}

impl<T> SkipOrQuit<T> for Result<T, SpreadsheetRecordImportError> {
    fn or_quit(self) -> RecordImportResult<T> {
        self.map_err(IncompleteOrCritical::Critical)
    }
    fn or_skip(self) -> RecordImportResult<T> {
        self.map_err(IncompleteOrCritical::Incomplete)
    }
}

impl From<SpreadsheetRecordImportError> for IncompleteOrCritical<SpreadsheetRecordImportError> {
    fn from(value: SpreadsheetRecordImportError) -> Self {
        IncompleteOrCritical::Critical(value)
    }
}

pub type ImportMatchResult = RecordImportResult<(AnyMatch, Vec<AnyPerformance>)>;
pub type ImportSongResult = RecordImportResult<AnySong>;

#[typetag::serde(tag = "game")]
pub trait Game: Debug {
    fn pretty_name(&self) -> &'static str;
    fn url_shortname(&self) -> &'static str;

    fn ask_for_performance_new(&self) -> Result<AnyPerformance, AskError> {
        unimplemented!()
    }

    fn create_match_and_performance_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut SpreadsheetContext) -> ImportMatchResult {
        Err(Critical(SpreadsheetRecordImportError::NotImplemented))
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut SpreadsheetContext) -> ImportSongResult {
        Err(Critical(SpreadsheetRecordImportError::NotImplemented))
    }
}

/// Get an instance of the [`Game`] trait based on the provided string ID of the game.
///
/// # Examples
/// ```
/// use scoretracker::data::game::game_instance_from_id;
///
/// let game = game_instance_from_id("yarg").unwrap();
/// assert_eq!(game.pretty_name(), "Yet Another Rhythm Game");
///
/// let game = game_instance_from_id("gh3").unwrap();
/// assert_eq!(game.pretty_name(), "Guitar Hero III: Legends of Rock");
///
/// let game = game_instance_from_id("nonexistent_game");
/// assert!(game.is_none());
/// ```
pub fn game_instance_from_id(game_id: &str) -> Option<Box<dyn Game>> {
    #[derive(Serialize)]
    struct GameIdentifier {
        pub game: String,
    }
    let game_identifier = GameIdentifier { game: game_id.to_string() };
    let json = serde_json::to_string(&game_identifier).unwrap();
    serde_json::from_str(&json).ok()
}
