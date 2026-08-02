//! Data structures for Guitar Hero Arcade.

use crate::data::game::Game;
use crate::spreadsheet::record::Record;
use crate::spreadsheet::{BadRecordError, IncompleteOrCritical::Continue, ParseSongRecordResult, context::Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GuitarHeroArcade;

#[typetag::serde(name = "gharcade")]
impl Game for GuitarHeroArcade {
    fn pretty_name(&self) -> &'static str {
        "Guitar Hero Arcade"
    }
    fn url_shortname(&self) -> &'static str {
        "gharcade"
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseSongRecordResult {
        Err(Continue(BadRecordError::NotImplemented)) // TODO
    }
}
