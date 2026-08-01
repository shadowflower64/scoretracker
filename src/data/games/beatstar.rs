//! Data structures for Beatstar.

use crate::data::game::{Game, ImportSongResult, IncompleteOrCritical::Incomplete, SpreadsheetContext};
use crate::spreadsheet::{RecordError, record::Record};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Beatstar;

#[typetag::serde(name = "beatstar")]
impl Game for Beatstar {
    fn pretty_name(&self) -> &'static str {
        "Beatstar"
    }
    fn url_shortname(&self) -> &'static str {
        "beatstar"
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut SpreadsheetContext) -> ImportSongResult {
        Err(Incomplete(RecordError::NotImplemented)) // TODO
    }
}
