//! Data structures for Hatsune Miku: Project DIVA Mega Mix+.

use crate::data::game::{Game, ImportSongResult, IncompleteOrCritical::Incomplete, SpreadsheetContext};
use crate::spreadsheet::{RecordError, record::Record};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectDIVAMegaMixPlus;

#[typetag::serde(name = "pjd_megamix_plus")]
impl Game for ProjectDIVAMegaMixPlus {
    fn pretty_name(&self) -> &'static str {
        "Hatsune Miku: Project DIVA Mega Mix+"
    }
    fn url_shortname(&self) -> &'static str {
        "pjd_megamix_plus"
    }

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut SpreadsheetContext) -> ImportSongResult {
        Err(Incomplete(RecordError::NotImplemented)) // TODO
    }
}
