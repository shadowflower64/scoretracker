pub mod field_path;
pub mod field_value;
pub mod record;

use crate::config::Config;
use crate::data::game::IncompleteOrCritical::{Critical, Incomplete};
use crate::data::game::song::AnySong;
use crate::data::game::{Game, SpreadsheetContext, game_instance_from_id};
use crate::data::library::database::LibraryDatabase;
use crate::data::scoreboard::r#match::AnyMatch;
use crate::data::scoreboard::performance::AnyPerformance;
use crate::data::scoreboard::player::PlayerDatabase;
use crate::spreadsheet::field_path::FieldPath;
use crate::spreadsheet::field_value::FieldValue;
use crate::spreadsheet::record::{Record, parse_records};
use crate::success;
use crate::util::filelocked::FileLockableData;
use crate::util::{file_ex, lockfile};
use crate::{info, log_fn_name, warn};
use calamine::Data;
use calamine::{Hyperlink, Ods, OdsError, Range, Reader, Xlsx, XlsxError, open_workbook};
use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::Europe::Warsaw;
use chrono_tz::Tz;
use std::error::Error;
use std::path::Path;
use thiserror::Error;

pub type SongList = Vec<AnySong>;

pub struct SpreadsheetImportResults {
    pub song_lists: Vec<SongList>,
    pub matches: Vec<AnyMatch>,
    pub performances: Vec<AnyPerformance>,
}

#[derive(Debug, Error)]
pub enum SpreadsheetRecordImportError {
    #[error("not implemented")]
    NotImplemented,
    #[error("not implemented yet")]
    NotImplementedYet,
    #[error("field not present: '{0}'")]
    FieldNotPresent(FieldPath),
    #[error("field '{0}' not a string: {1:?}")]
    NotAString(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not an int: {1:?}")]
    NotAnInt(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not an int subtype: {1:?}")]
    NotAnIntSubtype(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not a bool: {1:?}")]
    NotABool(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not a timestamp: {1:?}")]
    NotATimestamp(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not a date: {1:?}")]
    NotADate(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not a hyperlink: {1:?}")]
    NotAHyperlink(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not a valid enum variant: {1:?}")]
    NotAValidEnumVariant(FieldPath, String),
    #[error("field '{path}' date {naive} is ambiguous in timezone {tz} (date is in a fold): it could be from {earliest} to {latest}")]
    LocalDateIsAmbiguous {
        path: FieldPath,
        naive: NaiveDateTime,
        tz: Tz,
        earliest: DateTime<Utc>,
        latest: DateTime<Utc>,
    },
    #[error("field '{path}' date {naive} is in a gap in timezone {tz}")]
    LocalDateIsInGap { path: FieldPath, naive: NaiveDateTime, tz: Tz },
    #[error("player '{name}' does not exist")]
    PlayerDoesNotExist { name: String },
    #[error("proof with youtube id '{youtube_id}' does not exist")]
    ProofDoesNotExist { youtube_id: String },
    #[error("invalid youtube url: {url}")]
    InvalidYouTubeUrl { url: String }, // video ID not found
    #[error("{0}")]
    CustomMessage(String),
    #[error("{0}")]
    Custom(#[from] Box<dyn Error>),
}

#[derive(Debug, Error)]
pub enum SpreadsheetImportError {
    #[error("cannot open ods file: {0}")]
    CannotOpenOds(#[from] OdsError),
    #[error("cannot open xlsx file: {0}")]
    CannotOpenXlsx(#[from] XlsxError),
    #[error("unknown game id: {0}")]
    UnknownGame(String),
    #[error("invalid sheet name: {0}")]
    InvalidSheetName(String),
    #[error("invalid table type: {0}")]
    InvalidTableType(String),
    #[error("cannot read config: {0}")]
    CannotReadConfig(lockfile::Error), // TODO: this should actually be file_ex::Error, because the config is not open for writing
    #[error("cannot read player database: {0}")]
    CannotReadPlayerDatabase(file_ex::Error),
    #[error("cannot read library database: {0}")]
    CannotReadLibraryDatabase(lockfile::Error),
    #[error("cannot parse performance for game '{game_id}', row {row}: {error};\nrecord = {record}")]
    ParsePerformanceError {
        game_id: String,
        row: usize,
        record: Record,
        error: Box<SpreadsheetRecordImportError>,
    },
    #[error("cannot parse performance for game '{game_id}', row {row}: {error}")]
    ParsePerformanceErrorQuieter {
        game_id: String,
        row: usize,
        error: Box<SpreadsheetRecordImportError>,
    },
    #[error("cannot parse song for game '{game_id}', row {row}: {error};\nrecord = {record}")]
    ParseSongError {
        game_id: String,
        row: usize,
        record: Record,
        error: Box<SpreadsheetRecordImportError>,
    },
    #[error("cannot parse song for game '{game_id}', row {row}: {error}")]
    ParseSongErrorQuieter {
        game_id: String,
        row: usize,
        error: Box<SpreadsheetRecordImportError>,
    },
}

// TODO: rework these verbocity levels
pub const VERBOSE_COMPLETE: bool = false;
pub const VERBOSE_INCOMPLETE: bool = true;
pub const QUIETER_WARNINGS: bool = true;

fn import_org_spreadsheet_matches(
    game: Box<dyn Game>,
    game_id: &str,
    records: Vec<Record>,
    matches: &mut Vec<AnyMatch>,
    performances: &mut Vec<AnyPerformance>,
    ctx: &mut SpreadsheetContext,
) -> Result<(), SpreadsheetImportError> {
    log_fn_name!("import_org_spreadsheet_matches");

    let make_throwable = |i: usize, record: Record, e: SpreadsheetRecordImportError, quieter: bool| {
        if quieter {
            SpreadsheetImportError::ParsePerformanceErrorQuieter {
                game_id: game_id.to_owned(),
                row: i + 2,
                error: Box::new(e),
            }
        } else {
            SpreadsheetImportError::ParsePerformanceError {
                game_id: game_id.to_owned(),
                row: i + 2,
                record,
                error: Box::new(e),
            }
        }
    };

    for (i, record) in records.iter().enumerate() {
        match game.create_match_and_performance_from_spreadsheet_record(record, ctx) {
            Ok((match_data, performance_data)) => {
                if VERBOSE_COMPLETE {
                    success!(
                        "{game_id}:{} | match parsed successfully: {match_data:?} + {performance_data:?}",
                        i + 2
                    );
                } else {
                    success!("{game_id}:{} | match parsed successfully ", i + 2);
                }
                matches.push(match_data);
                performances.extend(performance_data);
            }
            Err(Incomplete(e)) => {
                if VERBOSE_INCOMPLETE {
                    let e = make_throwable(i, record.to_owned(), e, QUIETER_WARNINGS);
                    warn!("{game_id}:{} | incomplete match record: {e}; ignoring", i + 2);
                } else {
                    warn!("{game_id}:{} | incomplete match record; ignoring", i + 2);
                }
            }
            Err(Critical(e)) => {
                return Err(make_throwable(i, record.to_owned(), e, false));
            }
        }
    }
    Ok(())
}

fn import_org_spreadsheet_songs(
    game: Box<dyn Game>,
    game_id: &str,
    records: Vec<Record>,
    song_lists: &mut Vec<Vec<AnySong>>,
    ctx: &mut SpreadsheetContext,
) -> Result<(), SpreadsheetImportError> {
    log_fn_name!("import_org_spreadsheet_songs");

    let make_throwable = |i: usize, record: Record, e: SpreadsheetRecordImportError, quieter: bool| {
        if quieter {
            SpreadsheetImportError::ParseSongErrorQuieter {
                game_id: game_id.to_owned(),
                row: i + 2,
                error: Box::new(e),
            }
        } else {
            SpreadsheetImportError::ParseSongError {
                game_id: game_id.to_owned(),
                row: i + 2,
                record,
                error: Box::new(e),
            }
        }
    };

    let mut song_list: Vec<AnySong> = Vec::new();
    for (i, record) in records.iter().enumerate() {
        match game.create_song_from_spreadsheet_record(record, ctx) {
            Ok(song) => {
                if VERBOSE_COMPLETE {
                    success!("{game_id}:{} | song parsed successfully: {song:?}", i + 2);
                } else {
                    success!("{game_id}:{} | song parsed successfully", i + 2);
                }
                song_list.push(song);
            }
            Err(Incomplete(e)) => {
                if VERBOSE_INCOMPLETE {
                    let e = make_throwable(i, record.to_owned(), e, QUIETER_WARNINGS);
                    warn!("{game_id}:{} | incomplete song record: {e}; ignoring", i + 2);
                } else {
                    warn!("{game_id}:{} | incomplete song record; ignoring", i + 2);
                }
            }
            Err(Critical(e)) => {
                return Err(make_throwable(i, record.to_owned(), e, false));
            }
        }
    }
    song_lists.push(song_list);
    Ok(())
}

pub fn import_org_spreadsheet_generic<F>(
    mut worksheets: Vec<(String, Range<Data>)>,
    mut read_hyperlinks: F,
) -> Result<SpreadsheetImportResults, SpreadsheetImportError>
where
    F: FnMut(&str) -> Vec<Hyperlink>,
{
    log_fn_name!("import_org_spreadsheet_generic");

    let total_worksheets = worksheets.len();
    worksheets.retain(|(name, _)| name.starts_with("j."));
    let filtered_worksheets = worksheets.len();

    let names: Vec<_> = worksheets.iter().map(|(name, _)| name.to_string()).collect();
    info!("total worksheets: {total_worksheets}, filtered worksheets: {filtered_worksheets}, names: {names:?}");

    let mut song_lists = Vec::new();
    let mut matches = Vec::new();
    let mut performances = Vec::new();

    for (sheet_name, range) in worksheets {
        let sheet_name_split = FieldPath::from(&sheet_name);
        let table_type = sheet_name_split
            .0
            .get(1)
            .ok_or_else(|| SpreadsheetImportError::InvalidSheetName(sheet_name.to_owned()))?;
        let game_id = sheet_name_split
            .0
            .get(2)
            .ok_or_else(|| SpreadsheetImportError::InvalidSheetName(sheet_name.to_owned()))?;

        let game = game_instance_from_id(game_id.as_str()).ok_or_else(|| SpreadsheetImportError::UnknownGame(game_id.to_owned()))?;

        let config = Config::load().map_err(SpreadsheetImportError::CannotReadConfig)?;
        let player_database = PlayerDatabase::read_without_locking(config.player_database_path())
            .map_err(SpreadsheetImportError::CannotReadPlayerDatabase)?;
        let library_database = LibraryDatabase::lock_and_read(config.library_database_path(), None)
            .map_err(SpreadsheetImportError::CannotReadLibraryDatabase)?;
        let mut ctx = SpreadsheetContext {
            player_database: &player_database,
            library_database: &library_database,
            proofs_to_insert: Vec::new(),
            tz: Warsaw, // all legacy sheet times use Europe/Warsaw timezone
        };

        let records = parse_records(&sheet_name, range, read_hyperlinks(&sheet_name));
        // println!("{records:#?}");

        match table_type.as_str() {
            "matches" => {
                import_org_spreadsheet_matches(game, game_id, records, &mut matches, &mut performances, &mut ctx)?;
            }
            "songs" => {
                import_org_spreadsheet_songs(game, game_id, records, &mut song_lists, &mut ctx)?;
            }
            a => Err(SpreadsheetImportError::InvalidTableType(a.to_owned()))?,
        }
    }

    Ok(SpreadsheetImportResults {
        song_lists,
        matches,
        performances,
    })
}

pub fn import_org_spreadsheet_ods(ods_path: &Path) -> Result<SpreadsheetImportResults, SpreadsheetImportError> {
    log_fn_name!("import_org_spreadsheet_ods");
    info!("loading workbook from path: {ods_path:?}");
    let mut workbook: Ods<_> = open_workbook(ods_path)?;
    info!("loading workbook from path done");

    info!("fetching worksheets...");
    let worksheets = workbook.worksheets();
    info!("fetched worksheets successfully");

    import_org_spreadsheet_generic(worksheets, |_| Vec::new())
}

pub fn import_org_spreadsheet_xlsx(xlsx_path: &Path) -> Result<SpreadsheetImportResults, SpreadsheetImportError> {
    log_fn_name!("import_org_spreadsheet_xlsx");
    info!("loading workbook from path: {xlsx_path:?}");
    let mut workbook: Xlsx<_> = open_workbook(xlsx_path)?;
    info!("loading workbook from path done");

    info!("fetching worksheets...");
    let worksheets = workbook.worksheets();
    info!("fetched worksheets successfully");

    let read_hyperlinks = |name: &str| {
        log_fn_name!("import_org_spreadsheet_xlsx:read_hyperlinks");
        info!("reading hyperlinks for worksheet: '{name}'");
        let hyperlinks = workbook.hyperlinks_by_sheet_name(name).expect("todo");
        info!("reading hyperlinks for worksheet done");
        hyperlinks
    };
    import_org_spreadsheet_generic(worksheets, read_hyperlinks)
}
