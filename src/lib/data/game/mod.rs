pub mod song;

use crate::data::games::registered_games;
use crate::data::scoreboard::performance::AnyPerformance;
use crate::spreadsheet::ContinueOrQuit::Quit;
use crate::spreadsheet::context::Context;
use crate::spreadsheet::{BadRecordError, record::Record};
use crate::spreadsheet::{ParseMatchRecordResult, ParseSongRecordResult};
use crate::util::command_line::AskError;
use schemars::Schema;
use std::fmt::Debug;

#[typetag::serde(tag = "game")] // TODO: i don't think typetag is needed for games anymore; register_game!() macro along with the game_instance_from_id() function is better... the only problem rn is that IDs are not enforced as unique.
pub trait Game: Debug {
    fn identifier(&self) -> &'static str {
        self.typetag_name()
    }
    fn pretty_name(&self) -> &'static str;
    fn url_shortname(&self) -> &'static str;

    fn ask_for_performance_new(&self) -> Result<Box<AnyPerformance>, AskError> {
        unimplemented!("not implemented for game '{}'", self.identifier())
    }

    fn create_match_and_performance_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseMatchRecordResult {
        Err(Quit(BadRecordError::NotImplemented))
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseSongRecordResult {
        Err(Quit(BadRecordError::NotImplemented))
    }

    fn schema_name(&self) -> String {
        self.identifier().to_string()
    }

    fn schema(&self) -> Schema {
        unimplemented!("schema gen not implemented for game '{}'", self.identifier())
    }
}

pub type AnyGame = &'static (dyn Game + Send + Sync);

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
pub fn game_instance_from_id(game_id: &str) -> Option<AnyGame> {
    registered_games().iter().find(|x| x.identifier() == game_id).map(|v| &**v)
    // #[derive(Serialize)]
    // struct GameIdentifier {
    //     pub game: String,
    // }
    // let game_identifier = GameIdentifier { game: game_id.to_string() };
    // let json = serde_json::to_string(&game_identifier).unwrap();
    // serde_json::from_str(&json).ok()
}

/// Automatically implement common functions for Game impl.
///
/// Currently only implements [`Game::schema`].
// TODO: this should probably become an attribute macro eventually.
#[macro_export]
macro_rules! game_impl {
    () => {
        game_impl!(self);
    };
    ($module:tt) => {
        /// Generate a [`schemars::Schema`] for this game's types.
        fn schema(&self) -> schemars::Schema {
            use schemars::{JsonSchema, schema_for};
            // Dummy struct for generating a schema with multiple types at once
            #[derive(JsonSchema)]
            struct __ {
                __performance: $module::Performance,
                __match: $module::Match,
                //__song: $module::Song,
            }
            let schema = schema_for!(__);
            schema
        }
    };
}

/// Register the provided game struct into the global GAMES registry (distributed slice).
#[macro_export]
macro_rules! register_game {
    ($game:tt) => {
        #[linkme::distributed_slice($crate::data::games::GAMES)]
        fn create_game_instance() -> &'static (dyn Game + Send + Sync) {
            static GAME: $game = $game;
            &GAME
        }
    };
}
