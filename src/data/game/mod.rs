pub mod song;

use crate::data::game::IncompleteOrCritical::Critical;
use crate::data::game::song::AnySong;
use crate::data::library::database::{LibraryDatabase, LibraryEntry};
use crate::data::scoreboard::r#match::{AnyMatch, CommonMatchInfo};
use crate::data::scoreboard::performance::{AnyPerformance, CommonPerformanceInfo, PerformanceTrait};
use crate::data::scoreboard::player::{Player, PlayerDatabase};
use crate::spreadsheet::RecordErrorWithContext;
use crate::spreadsheet::{RecordError, record::Record};
use crate::util::command_line::AskError;
use crate::util::uuid::UuidString;
use crate::util::youtube_id;
use calamine::Hyperlink;
use chrono_tz::Tz;
use indexmap::IndexMap;
use serde::Serialize;
use std::fmt::Debug;
use uuid::Uuid;

pub struct SpreadsheetContext<'a> {
    pub player_database: &'a PlayerDatabase,
    pub library_database: &'a LibraryDatabase,
    pub proofs_to_insert: Vec<LibraryEntry>,
    pub tz: Tz,
    pub incomplete_match_records: Vec<RecordErrorWithContext>,
    pub incomplete_song_records: Vec<RecordErrorWithContext>,
}

impl SpreadsheetContext<'_> {
    pub fn find_player_by_name(&self, name: &str) -> Result<&Player, RecordError> {
        self.player_database
            .find_player_by_name(name)
            .ok_or_else(|| RecordError::PlayerDoesNotExist { name: name.to_owned() })
    }

    pub fn find_proof_by_youtube_id(&self, youtube_id: &str) -> Result<&LibraryEntry, RecordError> {
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

        Err(RecordError::ProofDoesNotExist {
            youtube_id: youtube_id.to_owned(),
        })
    }

    pub fn get_or_insert_proof_by_hyperlink(&mut self, hyperlink: &Hyperlink) -> Result<UuidString, RecordError> {
        let url = hyperlink.target.as_ref().expect("todo");
        let Some(youtube_id) = youtube_id(url) else {
            return Err(RecordError::InvalidYouTubeUrl { url: url.to_owned() });
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

    fn create_proof(&mut self, record: &Record) -> Result<Vec<UuidString>, RecordError> {
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

        Err(RecordError::NotAHyperlink(
            "video".into(),
            Box::new(record.field_value("video")?.to_owned()),
        ))
    }

    pub fn create_common_p(&mut self, record: &Record) -> Result<CommonPerformanceInfo, RecordError> {
        let comment = match record.field_value("comment") {
            Ok(value) => Some(
                value
                    .as_string()
                    .ok_or(RecordError::NotAString("comment".into(), Box::new(value.to_owned())))?
                    .to_owned(),
            ),
            Err(_) => None,
        };
        Ok(CommonPerformanceInfo {
            uuid: Uuid::now_v7().into(),
            player_uuid: self.find_player_by_name(&record.string("player")?)?.uuid,
            proof: self.create_proof(record)?,
            comment: comment,
            metadata: IndexMap::new(),
        })
    }

    pub fn create_common_m<P: PerformanceTrait>(&mut self, record: &Record, performances: &[&P]) -> RecordImportResult<CommonMatchInfo> {
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

    pub fn check_early_skip(&mut self, record: &Record) -> RecordImportResult<()> {
        record.timestamp("timestamp", self.tz).or_skip()?;
        Ok(())
    }
}

pub enum IncompleteOrCritical<E> {
    Incomplete(E),
    Critical(E),
}
pub type RecordImportResult<T> = Result<T, IncompleteOrCritical<RecordError>>;

pub trait SkipOrQuit<T> {
    fn or_skip(self) -> RecordImportResult<T>;
    fn or_quit(self) -> RecordImportResult<T>;
}

impl<T> SkipOrQuit<T> for Result<T, RecordError> {
    fn or_quit(self) -> RecordImportResult<T> {
        self.map_err(IncompleteOrCritical::Critical)
    }
    fn or_skip(self) -> RecordImportResult<T> {
        self.map_err(IncompleteOrCritical::Incomplete)
    }
}

impl From<RecordError> for IncompleteOrCritical<RecordError> {
    fn from(value: RecordError) -> Self {
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
        Err(Critical(RecordError::NotImplemented))
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut SpreadsheetContext) -> ImportSongResult {
        Err(Critical(RecordError::NotImplemented))
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
