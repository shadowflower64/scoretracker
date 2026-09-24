use calamine::Hyperlink;
use chrono_tz::Tz;

use crate::data::library::entry::Proof;
use crate::data::scoreboard::player::Player;
use crate::spreadsheet::record::Record;
use crate::spreadsheet::{BadRecordError, BadRecordErrorWithContext, ParseRecordResult, SkipOrQuit};
use crate::util::youtube_id;

pub struct Context {
    pub proofs_to_insert: Vec<Proof>,
    pub tz: Tz,
    pub ok_match_record_count: u32,
    pub ok_song_record_count: u32,
    pub throwaway_match_records: Vec<BadRecordErrorWithContext>,
    pub throwaway_song_records: Vec<BadRecordErrorWithContext>,
    pub fixable_match_records: Vec<BadRecordErrorWithContext>,
    pub fixable_song_records: Vec<BadRecordErrorWithContext>,
}

pub fn youtube_id_of_hyperlink(hyperlink: &Hyperlink) -> Result<String, BadRecordError> {
    let url = hyperlink
        .target
        .as_ref()
        .expect("hyperlink should have the target property set, purely internal hyperlinks are not supported");
    let Some(youtube_id) = youtube_id(url) else {
        return Err(BadRecordError::InvalidYouTubeUrl { url: url.to_owned() });
    };
    Ok(youtube_id)
}

pub fn youtube_ids_of_record(record: &Record) -> Result<Vec<String>, BadRecordError> {
    if let Some(string) = record.string_var("video")?
        && (string == ":(" || string == "-")
    {
        // `:(` => Proof got corrupted before it could be uploaded.
        // `-` => Proof never existed at all most likely.
        return Ok(Vec::new());
    }
    if let Some(hyperlink) = record.hyperlink_var("video")?
        && hyperlink.displayed_text == Some("YouTube".to_string())
    {
        let youtube_id = youtube_id_of_hyperlink(hyperlink)?;
        return Ok(vec![youtube_id]);
    }
    if record.is_empty("video")? {
        return Ok(Vec::new());
    }

    Err(BadRecordError::NotAHyperlink(
        "video".into(),
        Box::new(record.field_value("video")?.to_owned()),
    ))
}

impl Context {
    pub fn check_early_skip(&mut self, record: &Record) -> ParseRecordResult<()> {
        record.timestamp("timestamp", self.tz).or_skip()?;
        Ok(())
    }
}
