pub mod song;

use crate::data::game::song::AnySong;
use crate::data::scoreboard::r#match::AnyMatch;
use crate::data::scoreboard::performance::AnyPerformance;
use crate::data::scoreboard::player::{Player, PlayerDatabase};
use crate::spreadsheet::{Record, SpreadsheetRecordImportError};
use crate::util::command_line::AskError;
use serde::Serialize;
use std::fmt::Debug;

#[derive(Clone, Copy)]
pub struct SpreadsheetContext<'a> {
    pub player_database: &'a PlayerDatabase,
}

impl SpreadsheetContext<'_> {
    pub fn find_player_by_name(&self, name: &str) -> Result<&Player, SpreadsheetRecordImportError> {
        self.player_database
            .find_player_by_name(name)
            .ok_or_else(|| SpreadsheetRecordImportError::PlayerDoesNotExist { name: name.to_owned() })
    }
}

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
        _ctx: SpreadsheetContext,
    ) -> Result<(AnyMatch, Vec<AnyPerformance>), SpreadsheetRecordImportError> {
        Err(SpreadsheetRecordImportError::NotImplemented)
    }

    fn create_song_from_spreadsheet_record(
        &self,
        _record: &Record,
        _ctx: SpreadsheetContext,
    ) -> Result<AnySong, SpreadsheetRecordImportError> {
        Err(SpreadsheetRecordImportError::NotImplemented)
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
