use std::time::Duration;

use calamine::{Data, ExcelDateTime, Hyperlink};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, offset::LocalResult};
use chrono_tz::Tz;

use crate::spreadsheet::field_value::CellContents::{Empty, Filled};
use crate::spreadsheet::{BadRecordError, field_path::FieldPath};
use crate::util::timestamp::{NsDuration, NsTimestamp};
use crate::{debug, error, log_fn_name, log_should_print_debug, warn};

#[derive(Debug, Clone)]
pub enum FieldValue {
    String(String),
    Bool(bool), // unused
    Int(i64),   // unused
    Float(f64),
    DateTime(NsTimestamp), // unused
    Duration(NsDuration),
    DateOnlyNoTz(NaiveDate),
    DateTimeNoTz(NaiveDateTime),
    DateTimeWithMsNoTz(NaiveDateTime),
    Hyperlink(Hyperlink),
    InvalidFormula,
}

impl FieldValue {
    pub const ALLOW_FLOATS_AS_INTS: bool = true;
    pub const ALLOW_FLOATS_AS_BOOLS: bool = true;
    pub const ALLOW_INTS_AS_BOOLS: bool = true;

    pub fn as_string(&self) -> Option<&String> {
        match self {
            Self::String(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            FieldValue::Int(int) => Some(*int),
            FieldValue::Float(float) if *float == float.round() && Self::ALLOW_FLOATS_AS_INTS => Some(float.round() as i64),
            _ => None,
        }
    }

    pub fn as_int<T: TryFrom<i64>>(&self) -> Option<T> {
        self.as_i64().and_then(|x| x.try_into().ok())
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            FieldValue::Int(int) => Some(*int as f64),
            FieldValue::Float(float) => Some(*float),
            _ => None,
        }
    }

    pub fn as_float<T: TryFrom<f64>>(&self) -> Option<T> {
        self.as_f64().and_then(|x| x.try_into().ok())
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            FieldValue::Int(int) => Self::try_int_as_bool(*int),
            FieldValue::Float(float) => Self::try_float_as_bool(*float),
            FieldValue::Bool(bool) => Some(*bool),
            _ => None,
        }
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

    pub fn as_date(&self) -> Option<NaiveDate> {
        match self {
            FieldValue::DateOnlyNoTz(naive) => Some(*naive),
            _ => None,
        }
    }

    pub fn as_datetime(&self) -> Option<NaiveDateTime> {
        match self {
            FieldValue::DateTimeWithMsNoTz(naive) | FieldValue::DateTimeNoTz(naive) => Some(*naive),
            _ => None,
        }
    }

    pub fn try_as_timestamp(&self, path: FieldPath, tz: Tz) -> Result<NsTimestamp, BadRecordError> {
        match self {
            FieldValue::DateTimeWithMsNoTz(naive) | FieldValue::DateTimeNoTz(naive) => {
                let naive = *naive;
                match tz.from_local_datetime(&naive) {
                    LocalResult::Single(converted) => Ok(NsTimestamp::from(converted)),
                    LocalResult::Ambiguous(earliest, latest) => Err(BadRecordError::LocalDateIsAmbiguous {
                        naive,
                        path,
                        tz,
                        earliest: earliest.to_utc(),
                        latest: latest.to_utc(),
                    }),
                    LocalResult::None => Err(BadRecordError::LocalDateIsInGap { naive, path, tz }),
                }
            }
            FieldValue::DateTime(timestamp) => Ok(*timestamp),
            FieldValue::DateOnlyNoTz(_) => Err(BadRecordError::NotATimestamp(path, Box::new(self.to_owned()))), // this is too imprecise to use as a "timestamp"
            _ => Err(BadRecordError::NotATimestamp(path, Box::new(self.to_owned()))),
        }
    }

    pub fn as_hyperlink(&self) -> Option<&Hyperlink> {
        match self {
            FieldValue::Hyperlink(hyperlink) => Some(hyperlink),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CellContents {
    Empty,
    Filled(FieldValue),
}

impl CellContents {
    pub fn is_filled(&self) -> bool {
        matches!(&self, Self::Filled(_))
    }
    pub fn is_empty(&self) -> bool {
        matches!(&self, Self::Empty)
    }

    pub fn val(&self) -> Option<&FieldValue> {
        match self {
            Self::Empty => None,
            Self::Filled(a) => Some(a),
        }
    }
}

pub fn parse_cell_contents(cell: &Data, hyperlink: Option<&Hyperlink>, formula_mode: bool, log_prefix: Option<&str>) -> CellContents {
    log_fn_name!("parse_cell_contents");
    log_should_print_debug!(false);

    let log_prefix = log_prefix.map(|x| format!("{x}; ")).unwrap_or_default();
    let parse_excel_date_time = |cell: &ExcelDateTime| {
        let (year, month, day, hour, minute, second, millisecond) = cell.to_ymd_hms_milli();
        if cell.is_datetime() {
            if let Some(datetime) = NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
                .and_then(|x| x.and_hms_milli_opt(hour as u32, minute as u32, second as u32, millisecond as u32))
            {
                debug!(
                    "{log_prefix}parsing as naive datetime with ms, success: {}",
                    datetime.format("%Y-%m-%dT%H:%M:%S%.3f")
                );
                return Filled(FieldValue::DateTimeWithMsNoTz(datetime));
            } else {
                error!("{log_prefix}parsing as naive datetime with ms, fail: cannot create NaiveDate");
            }
        }
        if cell.is_duration() {
            let iso_duration =
                iso8601_duration::Duration::new(year as f32, month as f32, day as f32, hour as f32, minute as f32, second as f32);

            if let Some(duration) = iso_duration.to_std() {
                debug!("{log_prefix}parsing duration, success: {iso_duration} (millis: {millisecond})");
                let duration = duration + Duration::from_millis(millisecond as u64);
                return Filled(FieldValue::Duration(NsDuration::from(duration)));
            } else {
                error!("{log_prefix}parsing duration, fail: {iso_duration} (millis: {millisecond}): cannot convert to std duration");
            }
        }
        Empty
    };
    let parse_date_time = |cell: &str| {
        if cell.len() == "YYYY-MM-DD".len() {
            // Attempt to parse YYYY-MM-DD - date only, no timezone
            let result = NaiveDate::parse_from_str(cell, "%Y-%m-%d");
            if let Ok(date) = result {
                debug!("{log_prefix}parsing as naive date only, success: {date}");
                return Filled(FieldValue::DateOnlyNoTz(date));
            } else {
                warn!("{log_prefix}parsing as naive date only, fail: {result:?}");
            }
        }

        if cell.len() == "YYYY-MM-DDTHH:MM:SS".len() {
            // Attempt to parse YYYY-MM-DDTHH:MM:SS - date+time, no timezone
            let result = NaiveDateTime::parse_from_str(cell, "%Y-%m-%dT%H:%M:%S");
            if let Ok(datetime) = result {
                debug!("{log_prefix}parsing as naive datetime, success: {datetime}");
                if datetime.format("%H:%M:%S").to_string() == "00:00:00" || datetime.format("%H:%M:%S").to_string().len() != 8 {
                    panic!(
                        "there is no time in this datetime... so this cannot be stored as just a nstimestamp without extra information without losing any information."
                    );
                }
                return Filled(FieldValue::DateTimeNoTz(datetime));
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
                debug!(
                    "{log_prefix}parsing as naive datetime with ms, success: {}",
                    datetime.format("%Y-%m-%dT%H:%M:%S%.3f")
                );
                if datetime.format("%.3f").to_string() == ".000" || datetime.format("%.3f").to_string().len() != 4 {
                    panic!(
                        "there are no ms in this datetime... so this cannot be stored as just a nstimestamp without extra information without losing any information."
                    );
                }
                return Filled(FieldValue::DateTimeWithMsNoTz(datetime));
            } else {
                warn!("{log_prefix}parsing as naive datetime with ms, fail: {result:?}");
            }
        }

        let result = DateTime::parse_from_rfc3339(cell);
        if let Ok(datetime) = result {
            debug!("{log_prefix}parsing as rfc3339 datetime, success: {datetime}");
            return Filled(FieldValue::DateTime(NsTimestamp::from(datetime)));
        } else {
            warn!("{log_prefix}parsing as rfc3339 datetime, fail: {result:?}");
        }
        Empty
    };
    let parse_duration = |cell: &str| {
        let result = iso8601_duration::Duration::parse(cell);
        if let Ok(duration) = result {
            debug!("{log_prefix}parsing duration, success: {duration}");
            return Filled(FieldValue::Duration(NsDuration::from(duration.to_std().expect("todo"))));
        } else {
            warn!("{log_prefix}parsing duration, fail: {result:?}");
        }
        Empty
    };

    match (cell, hyperlink) {
        (_, Some(hyperlink)) => Filled(FieldValue::Hyperlink(hyperlink.clone())),
        (Data::String(a), _) => Filled(FieldValue::String(a.to_owned())),
        (Data::Bool(a), _) => Filled(FieldValue::Bool(*a)),
        (Data::Int(a), _) => Filled(FieldValue::Int(*a)),
        (Data::Float(a), _) => Filled(FieldValue::Float(*a)),
        (Data::DateTime(a), _) => parse_excel_date_time(a),
        (Data::DateTimeIso(a), _) => parse_date_time(a),
        (Data::DurationIso(a), _) => parse_duration(a),
        (Data::Empty, _) => Empty,
        (Data::Error(a), _) => {
            if !formula_mode {
                warn!("{log_prefix}cell with error read: {a:?}; treating as an InvalidFormula");
            }
            Filled(FieldValue::InvalidFormula)
        }
    }
}
