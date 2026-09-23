//! Placeholder data structures for testing purposes only.

use crate::data::game::Game;
use crate::data::scoreboard::r#match::MatchDetails;
use crate::data::scoreboard::performance::PerformanceDetails;
use crate::data::scoreboard::{r#match::Match, performance::Performance};
use crate::spreadsheet::ContinueOrQuit::Continue;
use crate::spreadsheet::context::Context;
use crate::spreadsheet::{BadRecordError, ParseMatchRecordResult, ParseSongRecordResult, SkipOrQuit};
use crate::{game_impl, register_game};
use crate::{spreadsheet::record::Record, util::command_line::AskError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PlaceholderMatchDetails {}

#[typetag::serde(name = "placeholder")]
impl MatchDetails for PlaceholderMatchDetails {
    fn sorting_key(&self) -> f64 {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PlaceholderPerformanceDetails {}

#[typetag::serde(name = "placeholder")]
impl PerformanceDetails for PlaceholderPerformanceDetails {
    fn sorting_key(&self) -> f64 {
        unimplemented!()
    }
    fn ask_for_performance_edit(&mut self) -> Result<(), AskError> {
        unimplemented!()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlaceholderGame;

#[typetag::serde(name = "placeholder")]
impl Game for PlaceholderGame {
    fn pretty_name(&self) -> &'static str {
        "Placeholder Game"
    }
    fn url_shortname(&self) -> &'static str {
        "placeholder"
    }

    fn create_match_and_performance_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseMatchRecordResult {
        let match_data = PlaceholderMatchDetails {};
        let performance_data = PlaceholderPerformanceDetails {};
        Ok((Box::new(match_data), vec![Box::new(performance_data)]))
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseSongRecordResult {
        Err(Continue(BadRecordError::NotImplemented)) // TODO
    }

    game_impl!();
}

register_game!(PlaceholderGame);
