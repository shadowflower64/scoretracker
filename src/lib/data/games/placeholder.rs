//! Placeholder data structures for a non-existent game, for easy testing.

use crate::{
    data::{
        game::Game,
        scoreboard::{r#match::MatchDetails, performance::PerformanceDetails},
        song::{chart::ChartDetails, chartset::ChartsetDetails},
    },
    game_impl, register_game,
    spreadsheet::{
        BadRecordError, ContinueOrQuit::Continue, ParseChartsetRecordResult, ParseMatchRecordResult, context::Context, record::Record,
    },
    util::command_line::AskError,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PlaceholderChartsetDetails {}

#[typetag::serde(name = "placeholder")]
impl ChartsetDetails for PlaceholderChartsetDetails {}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PlaceholderChartDetails {}

#[typetag::serde(name = "placeholder")]
impl ChartDetails for PlaceholderChartDetails {
    fn any_ref(&self) -> &dyn Any {
        self
    }
}

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

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseChartsetRecordResult {
        Err(Continue(BadRecordError::NotImplemented)) // TODO
    }

    game_impl!(PlaceholderMatchDetails, PlaceholderPerformanceDetails);
}

register_game!(PlaceholderGame);
