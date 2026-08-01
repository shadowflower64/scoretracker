//! Data structures for Phigros.

use crate::data::game::{Game, IncompleteOrCritical::Incomplete};
use crate::data::game::{ImportSongResult, SpreadsheetContext};
use crate::spreadsheet::RecordError;
use crate::spreadsheet::record::Record;
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

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut SpreadsheetContext) -> ImportSongResult {
        Err(Incomplete(RecordError::NotImplemented)) // TODO
    }
}
