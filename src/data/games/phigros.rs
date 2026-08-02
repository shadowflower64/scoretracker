//! Data structures for Phigros.

use crate::spreadsheet::IncompleteOrCritical::Continue;
use crate::spreadsheet::record::Record;
use crate::spreadsheet::{BadRecordError, ParseSongRecordResult};
use crate::{data::game::Game, spreadsheet::context::Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Phigros;

#[typetag::serde(name = "phigros")]
impl Game for Phigros {
    fn pretty_name(&self) -> &'static str {
        "Phigros"
    }
    fn url_shortname(&self) -> &'static str {
        "phigros"
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseSongRecordResult {
        Err(Continue(BadRecordError::NotImplemented)) // TODO
    }
}
