pub mod context;
pub mod field_path;
pub mod field_value;
pub mod record;

use crate::config::Config;
use crate::data::game::song::AnySong;
use crate::data::game::{Game, game_instance_from_id};
use crate::data::library::database::LibraryDatabase;
use crate::data::scoreboard::r#match::AnyMatch;
use crate::data::scoreboard::performance::AnyPerformance;
use crate::data::scoreboard::player::PlayerDatabase;
use crate::spreadsheet::ContinueOrQuit::{Continue, Quit};
use crate::spreadsheet::SpreadsheetImportError::{ParseMatchError, ParseSongError};
use crate::spreadsheet::context::Context;
use crate::spreadsheet::field_path::FieldPath;
use crate::spreadsheet::field_value::{CellContents, FieldValue};
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
use std::env::temp_dir;
use std::error::Error;
use std::path::Path;
use std::{fmt, fs};
use thiserror::Error;

pub type SongList = Vec<AnySong>;

pub enum ContinueOrQuit<E> {
    Continue(E),
    Quit(E),
}
pub type ParseRecordResult<T> = Result<T, ContinueOrQuit<BadRecordError>>;

pub trait SkipOrQuit<T> {
    fn or_skip(self) -> ParseRecordResult<T>;
    fn or_quit(self) -> ParseRecordResult<T>;
}

impl<T> SkipOrQuit<T> for Result<T, BadRecordError> {
    fn or_quit(self) -> ParseRecordResult<T> {
        self.map_err(ContinueOrQuit::Quit)
    }
    fn or_skip(self) -> ParseRecordResult<T> {
        self.map_err(ContinueOrQuit::Continue)
    }
}

impl From<BadRecordError> for ContinueOrQuit<BadRecordError> {
    fn from(value: BadRecordError) -> Self {
        ContinueOrQuit::Quit(value)
    }
}

pub type ParseMatchRecordResult = ParseRecordResult<(AnyMatch, Vec<AnyPerformance>)>;
pub type ParseSongRecordResult = ParseRecordResult<AnySong>;

pub struct SpreadsheetImportResults {
    pub song_lists: Vec<SongList>,
    pub matches: Vec<AnyMatch>,
    pub performances: Vec<AnyPerformance>,
}

#[derive(Debug, Error)]
pub enum BadRecordError {
    #[error("not implemented")]
    NotImplemented,
    #[error("not implemented yet")]
    NotImplementedYet,
    #[error("field not present: '{0}'")]
    FieldNotPresent(FieldPath),
    #[error("field '{0}' is empty")]
    CellIsEmpty(FieldPath),
    #[error("field '{0}' not a string: {1:?}")]
    NotAString(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not an int: {1:?}")]
    NotAnInt(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not an int subtype: {1:?}")]
    NotAnIntSubtype(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not an float: {1:?}")]
    NotAFloat(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not an float subtype: {1:?}")]
    NotAFloatSubtype(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not a bool: {1:?}")]
    NotABool(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not a timestamp: {1:?}")]
    NotATimestamp(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not a date: {1:?}")]
    NotADate(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not a hyperlink: {1:?}")]
    NotAHyperlink(FieldPath, Box<FieldValue>),
    #[error("field '{0}' not a valid enum variant for enum {1}: {2:?}")]
    NotAValidEnumVariant(FieldPath, String, String),
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
pub struct BadRecordErrorWithContext {
    pub game_id: String,
    pub row: usize,
    pub error: Box<BadRecordError>,
    pub record: Option<Record>,
}

impl fmt::Display for BadRecordErrorWithContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = format!("for game '{}', row {}: {}", self.game_id, self.row, self.error);
        if let Some(record) = &self.record {
            write!(f, "{str:60};\trecord = {record}",)
        } else {
            write!(f, "{}", str)
        }
    }
}

#[derive(Debug, Error)]
pub enum SpreadsheetImportError {
    #[error("cannot open ods file: {0}")]
    CannotOpenOds(#[from] OdsError),
    #[error("cannot open xlsx file: {0}")]
    CannotOpenXlsx(#[from] XlsxError),
    #[error("unknown game id: '{0}'")]
    UnknownGame(String),
    #[error("invalid sheet name: '{0}'")]
    InvalidSheetName(String),
    #[error("invalid table type: '{0}'")]
    InvalidTableType(String),
    #[error("cannot read config: {0}")]
    CannotReadConfig(file_ex::Error),
    #[error("cannot read player database: {0}")]
    CannotReadPlayerDatabase(file_ex::Error),
    #[error("cannot read library database: {0}")]
    CannotReadLibraryDatabase(lockfile::Error),
    #[error("cannot parse performance: {0}")]
    ParseMatchError(BadRecordErrorWithContext),
    #[error("cannot parse song: {0}")]
    ParseSongError(BadRecordErrorWithContext),
}

const VERBOSE_CORRECT: bool = false;
const VERBOSE_THROWAWAYS: bool = true;
const VERBOSE_FIXABLES: bool = true;
const PRINT_RECORD_FOR_THROWAWAYS: bool = false;
const PRINT_RECORD_FOR_FIXABLES: bool = true;
const STOP_FOR_FIXABLES: bool = false;

// idk anymore
fn throw_up(game_id: &str, i: usize, e: BadRecordError, record: &Record, show_record: bool) -> BadRecordErrorWithContext {
    BadRecordErrorWithContext {
        game_id: game_id.to_owned(),
        row: i + 2,
        record: show_record.then_some(record.to_owned()),
        error: Box::new(e),
    }
}

#[allow(clippy::too_many_arguments)]
fn import_org_spreadsheet_page<
    T: fmt::Debug,
    E: Fn(BadRecordErrorWithContext) -> SpreadsheetImportError,
    F: Fn(&Box<dyn Game>, &Record, &mut Context) -> ParseRecordResult<T>,
    G: FnMut(T, &mut Context),
    H: Fn(BadRecordErrorWithContext, &mut Context),
    I: Fn(BadRecordErrorWithContext, &mut Context),
>(
    game: Box<dyn Game>,
    game_id: &str,
    records: Vec<Record>,
    page_type: &str,
    make_err: E,
    parser_fn: F,
    mut on_success: G,
    on_throwaway: H,
    on_fixable: I,
    ctx: &mut Context,
) -> Result<(), SpreadsheetImportError> {
    log_fn_name!("import_org_spreadsheet_page");

    for (i, record) in records.iter().enumerate() {
        let row = i + 2;
        let throwaway = record.field("throwaway").and_then(CellContents::val).and_then(FieldValue::as_bool);

        match parser_fn(&game, record, ctx) {
            Ok(parser_output) => {
                if VERBOSE_CORRECT {
                    success!("{game_id}:{row} | {page_type} parsed successfully: {parser_output:?}");
                } else {
                    success!("{game_id}:{row} | {page_type} parsed successfully ");
                }
                on_success(parser_output, ctx);
            }
            Err(Continue(e)) => {
                if throwaway == Some(true) {
                    // throwaway
                    let e = throw_up(game_id, i, e, record, PRINT_RECORD_FOR_THROWAWAYS);
                    if VERBOSE_THROWAWAYS {
                        warn!("{game_id}:{row} | bad {page_type} record: {e}; but it was marked as throwaway; ignoring");
                    } else {
                        warn!("{game_id}:{row} | bad {page_type} record but it was marked as throwaway; ignoring");
                    }
                    on_throwaway(e, ctx);
                } else if throwaway == Some(false) {
                    // fixable
                    if STOP_FOR_FIXABLES {
                        warn!(
                            "{game_id}:{row} | bad {page_type} record and it was not marked as throwaway - it needs to be fixed; exiting"
                        );
                        return Err(make_err(throw_up(game_id, i, e, record, true)));
                    }

                    let e = throw_up(game_id, i, e, record, PRINT_RECORD_FOR_FIXABLES);
                    if VERBOSE_FIXABLES {
                        warn!(
                            "{game_id}:{row} | bad {page_type} record: {e}; and it was not marked as throwaway - it needs to be fixed; ignoring"
                        );
                    } else {
                        warn!(
                            "{game_id}:{row} | bad {page_type} record and it was not marked as throwaway - it needs to be fixed; ignoring"
                        );
                    }
                    on_fixable(e, ctx);
                } else {
                    warn!("{game_id}:{row} | bad {page_type} record and the throwaway marker is not defined; exiting");
                    return Err(make_err(throw_up(game_id, i, e, record, true)));
                }
            }
            Err(Quit(e)) => {
                let e = throw_up(game_id, i, e, record, true);
                return Err(make_err(e));
            }
        }
    }
    Ok(())
}

fn import_org_spreadsheet_matches(
    game: Box<dyn Game>,
    game_id: &str,
    records: Vec<Record>,
    matches: &mut Vec<AnyMatch>,
    performances: &mut Vec<AnyPerformance>,
    ctx: &mut Context,
) -> Result<(), SpreadsheetImportError> {
    import_org_spreadsheet_page(
        game,
        game_id,
        records,
        "match",
        ParseMatchError,
        |game, record, ctx| game.create_match_and_performance_from_spreadsheet_record(record, ctx),
        |(match_data, performance_data), ctx| {
            matches.push(match_data);
            performances.extend(performance_data);
            ctx.ok_match_record_count += 1;
        },
        |e, ctx| {
            ctx.throwaway_match_records.push(e);
        },
        |e, ctx| {
            ctx.fixable_match_records.push(e);
        },
        ctx,
    )
}

fn import_org_spreadsheet_songs(
    game: Box<dyn Game>,
    game_id: &str,
    records: Vec<Record>,
    song_lists: &mut Vec<Vec<AnySong>>,
    ctx: &mut Context,
) -> Result<(), SpreadsheetImportError> {
    let mut song_list = Vec::new();

    import_org_spreadsheet_page(
        game,
        game_id,
        records,
        "song",
        ParseSongError,
        |game, record, ctx| game.create_song_from_spreadsheet_record(record, ctx),
        |song, ctx| {
            song_list.push(song);
            ctx.ok_song_record_count += 1;
        },
        |e, ctx| {
            ctx.throwaway_song_records.push(e);
        },
        |e, ctx| {
            ctx.fixable_song_records.push(e);
        },
        ctx,
    )?;

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

    let config = Config::load().map_err(SpreadsheetImportError::CannotReadConfig)?;
    let player_database =
        PlayerDatabase::read_without_locking(config.player_database_path()).map_err(SpreadsheetImportError::CannotReadPlayerDatabase)?;
    let library_database =
        LibraryDatabase::lock_and_read(config.library_database_path(), None).map_err(SpreadsheetImportError::CannotReadLibraryDatabase)?;
    let mut ctx = Context {
        player_database: &player_database,
        library_database: &library_database,
        proofs_to_insert: Vec::new(),
        tz: Warsaw, // all legacy sheet times use Europe/Warsaw timezone
        ok_match_record_count: 0,
        ok_song_record_count: 0,
        throwaway_match_records: Vec::new(),
        throwaway_song_records: Vec::new(),
        fixable_match_records: Vec::new(),
        fixable_song_records: Vec::new(),
    };

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

        let records = parse_records(&sheet_name, range, read_hyperlinks(&sheet_name));

        match table_type.as_str() {
            "matches" => {
                import_org_spreadsheet_matches(game, game_id, records, &mut matches, &mut performances, &mut ctx)?;
            }
            "songs" => {
                let _ = import_org_spreadsheet_songs(game, game_id, records, &mut song_lists, &mut ctx); // TODO: ignore song errors for now
            }
            a => Err(SpreadsheetImportError::InvalidTableType(a.to_owned()))?,
        }
    }

    success!(
        "successfully imported {} match records and {} song records",
        ctx.ok_match_record_count,
        ctx.ok_song_record_count
    );
    success!(
        "{} match records thrown away, {} song records thrown away, {} fixable match records skipped, {} fixable song records skipped",
        ctx.throwaway_match_records.len(),
        ctx.throwaway_song_records.len(),
        ctx.fixable_match_records.len(),
        ctx.fixable_song_records.len()
    );

    let fixable_match_path = temp_dir().join("scoretracker/fixable_match_records.txt");
    fs::create_dir_all(fixable_match_path.parent().unwrap()).expect("could not create dirs");
    fs::write(
        &fixable_match_path,
        ctx.fixable_match_records
            .iter()
            .map(|line| format!("{line}\n"))
            .collect::<Vec<_>>()
            .join(""),
    )
    .expect("could not write to file");
    let fixable_song_path = temp_dir().join("scoretracker/fixable_song_records.txt");

    fs::write(
        &fixable_song_path,
        ctx.fixable_song_records
            .iter()
            .map(|line| format!("{line}\n"))
            .collect::<Vec<_>>()
            .join(""),
    )
    .expect("could not write to file");
    info!("fixables written to {fixable_match_path:?} and {fixable_song_path:?}");

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
        let hyperlinks = workbook
            .hyperlinks_by_sheet_name(name)
            .expect("the provided worksheet name should be a valid name of a worksheet in the spreadsheet");
        info!("reading hyperlinks for worksheet done");
        hyperlinks
    };
    import_org_spreadsheet_generic(worksheets, read_hyperlinks)
}
