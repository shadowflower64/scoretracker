use crate::scoreboard::r#match::AnyMatch;
use crate::scoreboard::performance::AnyPerformance;
use crate::songdb::song::AnySong;
use crate::spreadsheet::{Record, SpreadsheetRecordImportError};
use crate::util::command_line::AskError;
use serde::Serialize;
use std::fmt::Debug;

#[typetag::serde(tag = "game")]
pub trait Game: Debug {
    fn pretty_name(&self) -> &'static str;
    fn url_shortname(&self) -> &'static str;

    fn ask_for_performance_new(&self) -> Result<AnyPerformance, AskError> {
        unimplemented!()
    }

    fn create_match_and_performance_from_spreadsheet_record(
        &self,
        _record: &Record,
    ) -> Result<(AnyMatch, Vec<AnyPerformance>), SpreadsheetRecordImportError> {
        Err(SpreadsheetRecordImportError::NotImplemented)
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record) -> Result<AnySong, SpreadsheetRecordImportError> {
        Err(SpreadsheetRecordImportError::NotImplemented)
    }
}

/// Get an instance of the [`Game`] trait based on the provided string ID of the game.
///
/// # Examples
/// ```
/// use scoretracker::game::game_instance_from_id;
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
