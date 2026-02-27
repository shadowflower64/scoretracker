use crate::{scoreboard::performance::PerformanceTrait, songdb::song::SongTrait, util::cmd::AskError};
use serde::Serialize;
use std::{error::Error, fmt::Debug};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpreadsheetParseError {
    #[error("not implemented")]
    NotImplemented,
    #[error("not implemented yet")]
    NotImplementedYet,
    #[error("{0}")]
    CustomMessage(String),
    #[error("{0}")]
    Custom(Box<dyn Error>),
}

#[typetag::serde(tag = "game")]
pub trait Game: Debug {
    fn pretty_name(&self) -> &'static str;
    fn url_shortname(&self) -> &'static str;

    fn ask_for_performance_new(&self) -> Result<Box<dyn PerformanceTrait>, AskError> {
        unimplemented!()
    }

    fn create_performance_from_spreadsheet_row(
        &self,
        _row: Vec<(String, String)>,
    ) -> Result<Box<dyn PerformanceTrait>, SpreadsheetParseError> {
        Err(SpreadsheetParseError::NotImplemented)
    }

    fn create_song_from_spreadsheet_row(&self, _row: Vec<(String, String)>) -> Result<Box<dyn SongTrait>, SpreadsheetParseError> {
        Err(SpreadsheetParseError::NotImplemented)
    }
}

/// Get an instance of the [`Game`] trait based on the provided string ID of the game.
///
/// # Examples
/// ```
/// use scoretracker_core::game::game_instance_from_id;
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
