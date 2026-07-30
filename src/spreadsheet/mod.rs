use crate::config::Config;
use crate::data::game::song::AnySong;
use crate::data::game::{SpreadsheetContext, game_instance_from_id};
use crate::data::scoreboard::r#match::AnyMatch;
use crate::data::scoreboard::performance::AnyPerformance;
use crate::data::scoreboard::player::PlayerDatabase;
use crate::util::filelocked::FileLockableData;
use crate::util::uuid::UuidString;
use crate::util::{file_ex, lockfile};
use crate::{info, log_fn_name, success, util::timestamp::NsTimestamp, warn};
use calamine::Data::{self};
use calamine::{Ods, OdsError, Range, Reader, open_workbook};
use chrono::offset::LocalResult;
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::Warsaw;
use chrono_tz::Tz;
use indexmap::IndexMap;
use std::error::Error;
use std::fmt::Display;
use std::path::Path;
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
            return Err(SpreadsheetRecordImportError::NotAString(path, value.to_owned()));
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
            return Err(SpreadsheetRecordImportError::NotAString(path, value.to_owned()));
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
            _ => return Err(SpreadsheetRecordImportError::NotAnInt(path, value.to_owned())),
        };

        let Ok(requested_int) = int.try_into() else {
            return Err(SpreadsheetRecordImportError::NotAnIntSubtype(path, value.to_owned()));
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
                Ok(Self::try_int_as_bool(*int).ok_or_else(|| SpreadsheetRecordImportError::NotABool(path, value.to_owned()))?)
            }
            FieldValue::Float(float) => {
                Ok(Self::try_float_as_bool(*float).ok_or_else(|| SpreadsheetRecordImportError::NotABool(path, value.to_owned()))?)
            }
            FieldValue::Bool(bool) => Ok(*bool),
            _ => Err(SpreadsheetRecordImportError::NotABool(path, value.to_owned())),
        }
    }

    pub fn bool_or<K: Into<FieldPath>>(&self, key: K, value_if_empty: bool) -> Result<bool, SpreadsheetRecordImportError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Ok(value_if_empty);
        };
        match value {
            FieldValue::Int(int) => {
                Ok(Self::try_int_as_bool(*int).ok_or_else(|| SpreadsheetRecordImportError::NotABool(path, value.to_owned()))?)
            }
            FieldValue::Float(float) => {
                Ok(Self::try_float_as_bool(*float).ok_or_else(|| SpreadsheetRecordImportError::NotABool(path, value.to_owned()))?)
            }
            FieldValue::Bool(bool) => Ok(*bool),
            FieldValue::Empty => Ok(value_if_empty),
            _ => Err(SpreadsheetRecordImportError::NotABool(path, value.to_owned())),
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
            FieldValue::DateOnlyNoTz(_) => Err(SpreadsheetRecordImportError::NotATimestamp(path, value.to_owned())), // this is too imprecise to use as a "timestamp"
            _ => Err(SpreadsheetRecordImportError::NotATimestamp(path, value.to_owned())),
        }
    }

    pub fn date_only<K: Into<FieldPath>>(&self, key: K) -> Result<NaiveDate, SpreadsheetRecordImportError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(SpreadsheetRecordImportError::FieldNotPresent(path));
        };
        match value {
            FieldValue::DateOnlyNoTz(naive) => Ok(*naive),
            _ => Err(SpreadsheetRecordImportError::NotADate(path, value.to_owned())),
        }
    }

    pub fn hyperlink<K: Into<FieldPath>>(&self, key: K) -> Result<String, SpreadsheetRecordImportError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(SpreadsheetRecordImportError::FieldNotPresent(path));
        };
        // return Ok("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string());
        todo!("{:?}", value);
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

fn parse_cell_value(cell: &Data, log_prefix: Option<&str>) -> FieldValue {
    log_fn_name!("parse_cell_value");

    let log_prefix = log_prefix.map(|x| format!("{x}; ")).unwrap_or_default();
    let parse_date_time = |cell: &str| {
        if cell.len() == "YYYY-MM-DD".len() {
            // Attempt to parse YYYY-MM-DD - date only, no timezone
            let result = NaiveDate::parse_from_str(cell, "%Y-%m-%d");
            if let Ok(a) = result {
                success!("{log_prefix}parsing as naive date only, success: {a}");
                return FieldValue::DateOnlyNoTz(a);
            } else {
                warn!("{log_prefix}parsing as naive date only, fail: {result:?}");
            }
        }

        if cell.len() == "YYYY-MM-DDTHH:MM:SS".len() {
            // Attempt to parse YYYY-MM-DDTHH:MM:SS - date+time, no timezone
            let result = NaiveDateTime::parse_from_str(cell, "%Y-%m-%dT%H:%M:%S");
            if let Ok(a) = result {
                success!("{log_prefix}parsing as naive datetime, success: {a}");
                return FieldValue::DateTimeNoTz(a);
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
            if let Ok(a) = result {
                success!("{log_prefix}parsing as naive datetime with ms, success: {a}");
                return FieldValue::DateTimeWithMsNoTz(a);
            } else {
                warn!("{log_prefix}parsing as naive datetime with ms, fail: {result:?}");
            }
        }

        let result = DateTime::parse_from_rfc3339(cell);
        if let Ok(a) = result {
            success!("{log_prefix}parsing as rfc3339 datetime, success: {a}");
            return FieldValue::DateTime(NsTimestamp::from(a));
        } else {
            warn!("{log_prefix}parsing as rfc3339 datetime, fail: {result:?}");
        }
        FieldValue::Empty
    };
    let parse_duration = |cell: &str| {
        let result = iso8601_duration::Duration::parse(cell);
        if let Ok(a) = result {
            success!("{log_prefix}parsing duration, success: {a}");
            return FieldValue::Duration(NsTimestamp::from(a.to_std().expect("todo")));
        } else {
            warn!("{log_prefix}parsing duration, fail: {result:?}");
        }
        FieldValue::Empty
    };

    match cell {
        Data::String(a) => FieldValue::String(a.to_owned()),
        Data::Bool(a) => FieldValue::Bool(*a),
        Data::Int(a) => FieldValue::Int(*a),
        Data::Float(a) => FieldValue::Float(*a),
        Data::DateTimeIso(a) => parse_date_time(a),
        Data::DurationIso(a) => parse_duration(a),
        Data::Empty => FieldValue::Empty,

        a => unimplemented!("spreadsheet value: {a:?}"),
    }
}

pub type Records = Vec<Record>;

pub fn parse_records(sheet_name: &str, range: Range<Data>) -> Records {
    log_fn_name!("parse_records");

    info!("parsing sheet: {sheet_name}");

    let mut rows = range.rows();
    let Some(header_row) = rows.next() else {
        return Vec::new();
    };
    let header_row = header_row.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    let mut records = Vec::new();
    for (i, row) in rows.enumerate() {
        let mut record = Record::new();
        for (field_name, cell) in header_row.iter().zip(row) {
            // Skip fields that start with `.`
            if field_name.starts_with(".") {
                continue;
            }
            let prefix = format!("sheet: '{sheet_name}', row: {}, field: '{field_name}', value: '{cell}'", i + 2);
            let field_value = parse_cell_value(cell, Some(&prefix));
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

pub fn get_or_insert_proof_by_youtube_url(_youtube_url: &str) -> UuidString {
    // return Uuid::now_v7().into();
    todo!()
}

pub fn find_player_uuid_by_name(_player_name: &str) -> Option<UuidString> {
    // return Some(Uuid::now_v7().into());
    todo!()
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
    NotAString(FieldPath, FieldValue),
    #[error("field '{0}' not an int: {1:?}")]
    NotAnInt(FieldPath, FieldValue),
    #[error("field '{0}' not an int subtype: {1:?}")]
    NotAnIntSubtype(FieldPath, FieldValue),
    #[error("field '{0}' not a bool: {1:?}")]
    NotABool(FieldPath, FieldValue),
    #[error("field '{0}' not a timestamp: {1:?}")]
    NotATimestamp(FieldPath, FieldValue),
    #[error("field '{0}' not a date: {1:?}")]
    NotADate(FieldPath, FieldValue),
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
    #[error("{0}")]
    CustomMessage(String),
    #[error("{0}")]
    Custom(Box<dyn Error>),
}

#[derive(Debug, Error)]
pub enum SpreadsheetImportError {
    #[error("cannot open file: {0}")]
    CannotOpen(#[from] OdsError),
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

pub fn import_org_spreadsheet_ods(ods_path: &Path) -> Result<SpreadsheetImportResults, SpreadsheetImportError> {
    log_fn_name!("import_org_spreadsheet_ods");
    info!("loading workbook from path: {ods_path:?}");
    let mut workbook: Ods<_> = open_workbook(ods_path)?;
    info!("loading workbook from path done");

    let mut worksheets = workbook.worksheets();
    let total_worksheets = worksheets.len();
    worksheets.retain(|(name, _range)| name.starts_with("j."));
    let filtered_worksheets = worksheets.len();

    let names: Vec<_> = worksheets.iter().map(|(name, _range)| name.to_string()).collect();
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

        let records = parse_records(&sheet_name, range);
        // println!("{records:#?}");

        let game = game_instance_from_id(game_id.as_str()).ok_or_else(|| SpreadsheetImportError::UnknownGame(game_id.to_owned()))?;

        let config = Config::load().map_err(SpreadsheetImportError::CannotReadConfig)?;
        let player_database = PlayerDatabase::read_without_locking(config.player_database_path())
            .map_err(SpreadsheetImportError::CannotReadPlayerDatabase)?;
        let context = SpreadsheetContext {
            player_database: &player_database,
        };

        match table_type.as_str() {
            "matches" => {
                for (i, record) in records.iter().enumerate() {
                    let (match_data, performance_data) = game
                        .create_match_and_performance_from_spreadsheet_record(record, context)
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
                    let song = game.create_song_from_spreadsheet_record(record, context).map_err(|error| {
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
