//! Data structures for Hatsune Miku: Project DIVA Mega Mix+.

use crate::data::game::Game;
use crate::spreadsheet::ContinueOrQuit::Continue;
use crate::spreadsheet::{BadRecordError, ParseSongRecordResult, context::Context, record::Record};
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

    fn create_song_from_spreadsheet_record(&self, _record: &Record, _ctx: &mut Context) -> ParseSongRecordResult {
        Err(Continue(BadRecordError::NotImplemented)) // TODO
    }
}
