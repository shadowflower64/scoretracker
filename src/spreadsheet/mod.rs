use crate::config::Config;
use crate::data::game::song::AnySong;
use crate::data::game::{SpreadsheetContext, game_instance_from_id};
use crate::data::library::database::LibraryDatabase;
use crate::data::scoreboard::r#match::AnyMatch;
use crate::data::scoreboard::performance::AnyPerformance;
use crate::data::scoreboard::player::PlayerDatabase;
use crate::error;
use crate::util::filelocked::FileLockableData;
use crate::util::{file_ex, lockfile};
use crate::{info, log_fn_name, success, util::timestamp::NsTimestamp, warn};
use calamine::Data::{self};
use calamine::{ExcelDateTime, Hyperlink, Ods, OdsError, Range, Reader, Xlsx, XlsxError, open_workbook};
use chrono::offset::LocalResult;
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::Warsaw;
use chrono_tz::Tz;
use indexmap::IndexMap;
use std::error::Error;
use std::fmt::Display;
use std::path::Path;
use std::time::Duration;
use thiserror::Error;

/// A dot-separated field name.
///
/// This structure stores segments of a path for easier access.
///
/// # Examples
/// ```
/// # use scoretracker::spreadsheet::FieldPath;
/// assert_eq!(FieldPath::from("song_id"), FieldPath(vec!["song_id".to_string()]));
/// assert_eq!(FieldPath::from("chart.x.total_notes"), FieldPath(vec!["chart".to_string(), "x".to_string(), "total_notes".to_string()]));
/// assert_eq!(FieldPath(vec!["chart".to_string(), "x".to_string(), "total_notes".to_string()]).to_string().as_str(), "chart.x.total_notes");
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct FieldPath(pub Vec<String>);

impl<T: AsRef<str>> From<T> for FieldPath {
    fn from(value: T) -> Self {
        Self(value.as_ref().split(".").map(|x| x.to_owned()).collect())
    }
}

impl Display for FieldPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
}

#[derive(Debug, Clone)]
pub enum FieldValue {
    Empty,
    String(String),
    Bool(bool), // unused
    Int(i64),   // unused
    Float(f64),
    DateTime(NsTimestamp), // unused
    Duration(NsTimestamp),
    DateOnlyNoTz(NaiveDate),
    DateTimeNoTz(NaiveDateTime),
    DateTimeWithMsNoTz(NaiveDateTime),
    Hyperlink(Hyperlink),
}

#[derive(Default, Debug, Clone)]
pub struct Record(IndexMap<FieldPath, FieldValue>);

impl Record {
    pub const ALLOW_FLOATS_AS_INTS: bool = true;
    pub const ALLOW_FLOATS_AS_BOOLS: bool = true;
    pub const ALLOW_INTS_AS_BOOLS: bool = true;

    fn new() -> Self {
        Default::default()
    }

    pub fn string<K: Into<FieldPath>>(&self, key: K) -> Result<String, SpreadsheetRecordImportError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(SpreadsheetRecordImportError::FieldNotPresent(path));
        };
        let FieldValue::String(string) = value else {
            return Err(SpreadsheetRecordImportError::NotAString(path, Box::new(value.to_owned())));
        };

        Ok(string.to_owned())
    }

    pub fn string_opt<K: Into<FieldPath>>(&self, key: K) -> Result<Option<String>, SpreadsheetRecordImportError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else { return Ok(None) };
        if matches!(value, FieldValue::Empty) {
            return Ok(None);
        }
        let FieldValue::String(string) = value else {
            return Err(SpreadsheetRecordImportError::NotAString(path, Box::new(value.to_owned())));
        };

        Ok(Some(string.to_owned()))
    }

    pub fn int<T: TryFrom<i64>, K: Into<FieldPath>>(&self, key: K) -> Result<T, SpreadsheetRecordImportError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(SpreadsheetRecordImportError::FieldNotPresent(path));
        };
        let int = match value {
            FieldValue::Int(int) => *int,
            FieldValue::Float(float) if *float == float.round() && Self::ALLOW_FLOATS_AS_INTS => float.round() as i64,
            _ => return Err(SpreadsheetRecordImportError::NotAnInt(path, Box::new(value.to_owned()))),
        };

        let Ok(requested_int) = int.try_into() else {
            return Err(SpreadsheetRecordImportError::NotAnIntSubtype(path, Box::new(value.to_owned())));
        };

        Ok(requested_int)
    }

    fn try_int_as_bool(int: i64) -> Option<bool> {
        if Self::ALLOW_INTS_AS_BOOLS {
            if int == 0 {
                Some(false)
            } else if int == 1 {
                Some(true)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn try_float_as_bool(float: f64) -> Option<bool> {
        if float == float.round() && Self::ALLOW_FLOATS_AS_BOOLS {
            let int = float.round() as i64;
            if int == 0 {
                Some(false)
            } else if int == 1 {
                Some(true)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn bool<K: Into<FieldPath>>(&self, key: K) -> Result<bool, SpreadsheetRecordImportError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(SpreadsheetRecordImportError::FieldNotPresent(path));
        };
        match value {
            FieldValue::Int(int) => {
                Ok(Self::try_int_as_bool(*int).ok_or_else(|| SpreadsheetRecordImportError::NotABool(path, Box::new(value.to_owned())))?)
            }
            FieldValue::Float(float) => {
                Ok(Self::try_float_as_bool(*float)
                    .ok_or_else(|| SpreadsheetRecordImportError::NotABool(path, Box::new(value.to_owned())))?)
            }
            FieldValue::Bool(bool) => Ok(*bool),
            _ => Err(SpreadsheetRecordImportError::NotABool(path, Box::new(value.to_owned()))),
        }
    }

    pub fn bool_or<K: Into<FieldPath>>(&self, key: K, value_if_empty: bool) -> Result<bool, SpreadsheetRecordImportError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Ok(value_if_empty);
        };
        match value {
            FieldValue::Int(int) => {
                Ok(Self::try_int_as_bool(*int).ok_or_else(|| SpreadsheetRecordImportError::NotABool(path, Box::new(value.to_owned())))?)
            }
            FieldValue::Float(float) => {
                Ok(Self::try_float_as_bool(*float)
                    .ok_or_else(|| SpreadsheetRecordImportError::NotABool(path, Box::new(value.to_owned())))?)
            }
            FieldValue::Bool(bool) => Ok(*bool),
            FieldValue::Empty => Ok(value_if_empty),
            _ => Err(SpreadsheetRecordImportError::NotABool(path, Box::new(value.to_owned()))),
        }
    }

    pub fn timestamp<K: Into<FieldPath>>(&self, key: K) -> Result<NsTimestamp, SpreadsheetRecordImportError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(SpreadsheetRecordImportError::FieldNotPresent(path));
        };
        match value {
            FieldValue::DateTimeWithMsNoTz(naive) | FieldValue::DateTimeNoTz(naive) => {
                let naive = *naive;
                let tz = Warsaw; // all legacy sheet times use Europe/Warsaw timezone
                match tz.from_local_datetime(&naive) {
                    LocalResult::Single(converted) => Ok(NsTimestamp::from(converted)),
                    LocalResult::Ambiguous(earliest, latest) => Err(SpreadsheetRecordImportError::LocalDateIsAmbiguous {
                        naive,
                        path,
                        tz,
                        earliest: earliest.to_utc(),
                        latest: latest.to_utc(),
                    }),
                    LocalResult::None => Err(SpreadsheetRecordImportError::LocalDateIsInGap { naive, path, tz }),
                }
            }
            FieldValue::DateTime(timestamp) => Ok(*timestamp),
            FieldValue::DateOnlyNoTz(_) => Err(SpreadsheetRecordImportError::NotATimestamp(path, Box::new(value.to_owned()))), // this is too imprecise to use as a "timestamp"
            _ => Err(SpreadsheetRecordImportError::NotATimestamp(path, Box::new(value.to_owned()))),
        }
    }

    pub fn date_only<K: Into<FieldPath>>(&self, key: K) -> Result<NaiveDate, SpreadsheetRecordImportError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(SpreadsheetRecordImportError::FieldNotPresent(path));
        };
        match value {
            FieldValue::DateOnlyNoTz(naive) => Ok(*naive),
            _ => Err(SpreadsheetRecordImportError::NotADate(path, Box::new(value.to_owned()))),
        }
    }

    pub fn hyperlink<K: Into<FieldPath>>(&self, key: K) -> Result<Hyperlink, SpreadsheetRecordImportError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(SpreadsheetRecordImportError::FieldNotPresent(path));
        };
        let FieldValue::Hyperlink(hyperlink) = value else {
            return Err(SpreadsheetRecordImportError::NotAHyperlink(path, Box::new(value.to_owned())));
        };

        Ok(hyperlink.to_owned())
    }

    pub fn hyperlink_opt<K: Into<FieldPath>>(&self, key: K) -> Result<Option<Hyperlink>, SpreadsheetRecordImportError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else { return Ok(None) };
        if matches!(value, FieldValue::Empty) {
            return Ok(None);
        }
        let FieldValue::Hyperlink(hyperlink) = value else {
            return Err(SpreadsheetRecordImportError::NotAHyperlink(path, Box::new(value.to_owned())));
        };

        Ok(Some(hyperlink.to_owned()))
    }
}

impl AsRef<IndexMap<FieldPath, FieldValue>> for Record {
    fn as_ref(&self) -> &IndexMap<FieldPath, FieldValue> {
        &self.0
    }
}
impl AsMut<IndexMap<FieldPath, FieldValue>> for Record {
    fn as_mut(&mut self) -> &mut IndexMap<FieldPath, FieldValue> {
        &mut self.0
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        write!(f, "<Record: ")?;
        for (key, value) in &self.0 {
            if !first {
                write!(f, ", ")?;
            }
            first = false;
            write!(f, "{key} = {value:?}")?;
        }
        write!(f, ">")?;
        Ok(())
    }
}

fn parse_cell_value(cell: &Data, hyperlink: Option<&Hyperlink>, formula_mode: bool, log_prefix: Option<&str>) -> FieldValue {
    log_fn_name!("parse_cell_value");

    let log_prefix = log_prefix.map(|x| format!("{x}; ")).unwrap_or_default();
    let parse_excel_date_time = |cell: &ExcelDateTime| {
        let (year, month, day, hour, minute, second, millisecond) = cell.to_ymd_hms_milli();
        if cell.is_datetime() {
            if let Some(datetime) = NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
                .and_then(|x| x.and_hms_milli_opt(hour as u32, minute as u32, second as u32, millisecond as u32))
            {
                success!(
                    "{log_prefix}parsing as naive datetime with ms, success: {}",
                    datetime.format("%Y-%m-%dT%H:%M:%S%.3f")
                );
                return FieldValue::DateTimeWithMsNoTz(datetime);
            } else {
                error!("{log_prefix}parsing as naive datetime with ms, fail: cannot create NaiveDate");
            }
        }
        if cell.is_duration() {
            let iso_duration =
                iso8601_duration::Duration::new(year as f32, month as f32, day as f32, hour as f32, minute as f32, second as f32);

            if let Some(duration) = iso_duration.to_std() {
                success!("{log_prefix}parsing duration, success: {iso_duration} (millis: {millisecond})");
                let duration = duration + Duration::from_millis(millisecond as u64);
                return FieldValue::Duration(NsTimestamp::from(duration));
            } else {
                error!("{log_prefix}parsing duration, fail: {iso_duration} (millis: {millisecond}): cannot convert to std duration");
            }
        }
        FieldValue::Empty
    };
    let parse_date_time = |cell: &str| {
        if cell.len() == "YYYY-MM-DD".len() {
            // Attempt to parse YYYY-MM-DD - date only, no timezone
            let result = NaiveDate::parse_from_str(cell, "%Y-%m-%d");
            if let Ok(date) = result {
                success!("{log_prefix}parsing as naive date only, success: {date}");
                return FieldValue::DateOnlyNoTz(date);
            } else {
                warn!("{log_prefix}parsing as naive date only, fail: {result:?}");
            }
        }

        if cell.len() == "YYYY-MM-DDTHH:MM:SS".len() {
            // Attempt to parse YYYY-MM-DDTHH:MM:SS - date+time, no timezone
            let result = NaiveDateTime::parse_from_str(cell, "%Y-%m-%dT%H:%M:%S");
            if let Ok(datetime) = result {
                success!("{log_prefix}parsing as naive datetime, success: {datetime}");
                if datetime.format("%H:%M:%S").to_string() == "00:00:00" || datetime.format("%H:%M:%S").to_string().len() != 8 {
                    panic!(
                        "there is no time in this datetime... so this cannot be stored as just a nstimestamp without extra information without losing any information."
                    );
                }
                return FieldValue::DateTimeNoTz(datetime);
            } else {
                warn!("{log_prefix}parsing as naive datetime, fail: {result:?}");
            }
        }

        if cell.len() == "YYYY-MM-DDTHH:MM:SS.m".len()
            || cell.len() == "YYYY-MM-DDTHH:MM:SS.mm".len()
            || cell.len() == "YYYY-MM-DDTHH:MM:SS.mmm".len()
        {
            // Attempt to parse YYYY-MM-DDTHH:MM:SS.mmm - date+time+ms, no timezone
            let result = NaiveDateTime::parse_from_str(cell, "%Y-%m-%dT%H:%M:%S%.f");
            if let Ok(datetime) = result {
                success!(
                    "{log_prefix}parsing as naive datetime with ms, success: {}",
                    datetime.format("%Y-%m-%dT%H:%M:%S%.3f")
                );
                if datetime.format("%.3f").to_string() == ".000" || datetime.format("%.3f").to_string().len() != 4 {
                    panic!(
                        "there are no ms in this datetime... so this cannot be stored as just a nstimestamp without extra information without losing any information."
                    );
                }
                return FieldValue::DateTimeWithMsNoTz(datetime);
            } else {
                warn!("{log_prefix}parsing as naive datetime with ms, fail: {result:?}");
            }
        }

        let result = DateTime::parse_from_rfc3339(cell);
        if let Ok(datetime) = result {
            success!("{log_prefix}parsing as rfc3339 datetime, success: {datetime}");
            return FieldValue::DateTime(NsTimestamp::from(datetime));
        } else {
            warn!("{log_prefix}parsing as rfc3339 datetime, fail: {result:?}");
        }
        FieldValue::Empty
    };
    let parse_duration = |cell: &str| {
        let result = iso8601_duration::Duration::parse(cell);
        if let Ok(duration) = result {
            success!("{log_prefix}parsing duration, success: {duration}");
            return FieldValue::Duration(NsTimestamp::from(duration.to_std().expect("todo")));
        } else {
            warn!("{log_prefix}parsing duration, fail: {result:?}");
        }
        FieldValue::Empty
    };

    match (cell, hyperlink) {
        (_, Some(hyperlink)) => FieldValue::Hyperlink(hyperlink.clone()),
        (Data::String(a), _) => FieldValue::String(a.to_owned()),
        (Data::Bool(a), _) => FieldValue::Bool(*a),
        (Data::Int(a), _) => FieldValue::Int(*a),
        (Data::Float(a), _) => FieldValue::Float(*a),
        (Data::DateTime(a), _) => parse_excel_date_time(a),
        (Data::DateTimeIso(a), _) => parse_date_time(a),
        (Data::DurationIso(a), _) => parse_duration(a),
        (Data::Empty, _) => FieldValue::Empty,
        (Data::Error(a), _) => {
            if !formula_mode {
                warn!("{log_prefix}cell with error read: {a:?}; treating as an empty cell");
            }
            FieldValue::Empty
        }
    }
}

pub type Records = Vec<Record>;

pub fn parse_records(sheet_name: &str, range: Range<Data>, hyperlinks: Vec<Hyperlink>) -> Records {
    log_fn_name!("parse_records");

    info!("parsing sheet: {sheet_name}");
    let mut rows = range.rows();
    let Some(header_row) = rows.next() else {
        return Vec::new();
    };
    let header_row = header_row.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    let mut records = Vec::new();
    for (y, row) in rows.enumerate() {
        let mut record = Record::new();
        for (x, (field_name, cell)) in header_row.iter().zip(row).enumerate() {
            // Skip fields that start with `.`
            if field_name.starts_with('.') {
                continue;
            }
            // Fields that start with `$` contain formulas, which may or may not work. If they don't work, ignore the errors.
            // We don't really care about the formula cells; we will have our own functions for calculating those values.
            let formula_mode = field_name.starts_with('$');
            let hyperlink = hyperlinks
                .iter()
                .find(|hyperlink| hyperlink.range.contains((y + 1) as u32, x as u32));
            let prefix = format!("sheet: '{sheet_name}', row: {}, field: '{field_name}', value: '{cell}'", y + 2);
            let field_value = parse_cell_value(cell, hyperlink, formula_mode, Some(&prefix));
            record.as_mut().insert(field_name.into(), field_value);
        }
        records.push(record);
    }
    records
}

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
    Custom(Box<dyn Error>),
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
    #[error("could not create performance for game '{game_id}' from spreadsheet row {row}: {error};\nrecord = {record}")]
    ParsePerformanceError {
        game_id: String,
        row: usize,
        record: Record,
        error: Box<SpreadsheetRecordImportError>,
    },
    #[error("could not create song for game '{game_id}' from spreadsheet row {row}: {error};\nrecord = {record}")]
    ParseSongError {
        game_id: String,
        row: usize,
        record: Record,
        error: Box<SpreadsheetRecordImportError>,
    },
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
        if sheet_name != "j.matches.adofai" {
            continue; // TODO: DEBUG
        }
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
        let mut context = SpreadsheetContext {
            player_database: &player_database,
            library_database: &library_database,
            proofs_to_insert: Vec::new(),
        };

        let records = parse_records(&sheet_name, range, read_hyperlinks(&sheet_name));
        // println!("{records:#?}");

        match table_type.as_str() {
            "matches" => {
                for (i, record) in records.iter().enumerate() {
                    let (match_data, performance_data) = game
                        .create_match_and_performance_from_spreadsheet_record(record, &mut context)
                        .map_err(|error| SpreadsheetImportError::ParsePerformanceError {
                            game_id: game_id.to_owned(),
                            row: i + 2,
                            record: record.to_owned(),
                            error: Box::new(error),
                        })?;
                    matches.push(match_data);
                    performances.extend(performance_data);
                }
            }
            "songs" => {
                let mut song_list = Vec::new();
                for (i, record) in records.iter().enumerate() {
                    let song = game.create_song_from_spreadsheet_record(record, &mut context).map_err(|error| {
                        SpreadsheetImportError::ParseSongError {
                            game_id: game_id.to_owned(),
                            row: i + 2,
                            record: record.to_owned(),
                            error: Box::new(error),
                        }
                    })?;
                    song_list.push(song)
                }
                song_lists.push(song_list);
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
        info!("reading hyperlinks for worksheet '{name}'...");
        let hyperlinks = workbook.hyperlinks_by_sheet_name(name).expect("todo");
        info!("reading hyperlinks for worksheet '{name}'... done");
        hyperlinks
    };
    import_org_spreadsheet_generic(worksheets, read_hyperlinks)
}
