use crate::spreadsheet::field_value::parse_cell_value;
use crate::spreadsheet::{FieldPath, FieldValue, RecordError};
use crate::util::timestamp::NsTimestamp;
use crate::{info, log_fn_name};
use calamine::{Data, Hyperlink, Range};
use chrono::TimeZone;
use chrono::{NaiveDate, offset::LocalResult};
use chrono_tz::Tz;
use indexmap::IndexMap;
use std::fmt::Display;

#[derive(Default, Debug, Clone)]
pub struct Record(IndexMap<FieldPath, FieldValue>);

impl Record {
    pub const ALLOW_FLOATS_AS_INTS: bool = true;
    pub const ALLOW_FLOATS_AS_BOOLS: bool = true;
    pub const ALLOW_INTS_AS_BOOLS: bool = true;

    fn new() -> Self {
        Default::default()
    }

    pub fn field<K: Into<FieldPath>>(&self, key: K) -> Result<&FieldValue, RecordError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(RecordError::FieldNotPresent(path));
        };
        Ok(value)
    }

    pub fn field_opt<K: Into<FieldPath>>(&self, key: K) -> Option<&FieldValue> {
        let path = key.into();

        let value = self.0.get(&path)?;
        if matches!(value, FieldValue::Empty) {
            return None;
        }

        Some(value)
    }

    pub fn string<K: Into<FieldPath>>(&self, key: K) -> Result<String, RecordError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(RecordError::FieldNotPresent(path));
        };
        let FieldValue::String(string) = value else {
            return Err(RecordError::NotAString(path, Box::new(value.to_owned())));
        };

        Ok(string.to_owned())
    }

    pub fn string_opt<K: Into<FieldPath>>(&self, key: K) -> Result<Option<String>, RecordError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else { return Ok(None) };
        if matches!(value, FieldValue::Empty) {
            return Ok(None);
        }
        let FieldValue::String(string) = value else {
            return Err(RecordError::NotAString(path, Box::new(value.to_owned())));
        };

        Ok(Some(string.to_owned()))
    }

    pub fn int<T: TryFrom<i64>, K: Into<FieldPath>>(&self, key: K) -> Result<T, RecordError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(RecordError::FieldNotPresent(path));
        };
        let int = match value {
            FieldValue::Int(int) => *int,
            FieldValue::Float(float) if *float == float.round() && Self::ALLOW_FLOATS_AS_INTS => float.round() as i64,
            _ => return Err(RecordError::NotAnInt(path, Box::new(value.to_owned()))),
        };

        let Ok(requested_int) = int.try_into() else {
            return Err(RecordError::NotAnIntSubtype(path, Box::new(value.to_owned())));
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

    pub fn bool<K: Into<FieldPath>>(&self, key: K) -> Result<bool, RecordError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(RecordError::FieldNotPresent(path));
        };
        match value {
            FieldValue::Int(int) => Ok(Self::try_int_as_bool(*int).ok_or_else(|| RecordError::NotABool(path, Box::new(value.to_owned())))?),
            FieldValue::Float(float) => {
                Ok(Self::try_float_as_bool(*float).ok_or_else(|| RecordError::NotABool(path, Box::new(value.to_owned())))?)
            }
            FieldValue::Bool(bool) => Ok(*bool),
            _ => Err(RecordError::NotABool(path, Box::new(value.to_owned()))),
        }
    }

    pub fn bool_or<K: Into<FieldPath>>(&self, key: K, value_if_empty: bool) -> Result<bool, RecordError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Ok(value_if_empty);
        };
        match value {
            FieldValue::Int(int) => Ok(Self::try_int_as_bool(*int).ok_or_else(|| RecordError::NotABool(path, Box::new(value.to_owned())))?),
            FieldValue::Float(float) => {
                Ok(Self::try_float_as_bool(*float).ok_or_else(|| RecordError::NotABool(path, Box::new(value.to_owned())))?)
            }
            FieldValue::Bool(bool) => Ok(*bool),
            FieldValue::Empty => Ok(value_if_empty),
            _ => Err(RecordError::NotABool(path, Box::new(value.to_owned()))),
        }
    }

    pub fn timestamp<K: Into<FieldPath>>(&self, key: K, tz: Tz) -> Result<NsTimestamp, RecordError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(RecordError::FieldNotPresent(path));
        };
        match value {
            FieldValue::DateTimeWithMsNoTz(naive) | FieldValue::DateTimeNoTz(naive) => {
                let naive = *naive;
                match tz.from_local_datetime(&naive) {
                    LocalResult::Single(converted) => Ok(NsTimestamp::from(converted)),
                    LocalResult::Ambiguous(earliest, latest) => Err(RecordError::LocalDateIsAmbiguous {
                        naive,
                        path,
                        tz,
                        earliest: earliest.to_utc(),
                        latest: latest.to_utc(),
                    }),
                    LocalResult::None => Err(RecordError::LocalDateIsInGap { naive, path, tz }),
                }
            }
            FieldValue::DateTime(timestamp) => Ok(*timestamp),
            FieldValue::DateOnlyNoTz(_) => Err(RecordError::NotATimestamp(path, Box::new(value.to_owned()))), // this is too imprecise to use as a "timestamp"
            _ => Err(RecordError::NotATimestamp(path, Box::new(value.to_owned()))),
        }
    }

    pub fn date_only<K: Into<FieldPath>>(&self, key: K) -> Result<NaiveDate, RecordError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(RecordError::FieldNotPresent(path));
        };
        match value {
            FieldValue::DateOnlyNoTz(naive) => Ok(*naive),
            _ => Err(RecordError::NotADate(path, Box::new(value.to_owned()))),
        }
    }

    pub fn hyperlink<K: Into<FieldPath>>(&self, key: K) -> Result<Hyperlink, RecordError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(RecordError::FieldNotPresent(path));
        };
        let FieldValue::Hyperlink(hyperlink) = value else {
            return Err(RecordError::NotAHyperlink(path, Box::new(value.to_owned())));
        };

        Ok(hyperlink.to_owned())
    }

    pub fn hyperlink_opt<K: Into<FieldPath>>(&self, key: K) -> Result<Option<Hyperlink>, RecordError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else { return Ok(None) };
        if matches!(value, FieldValue::Empty) {
            return Ok(None);
        }
        let FieldValue::Hyperlink(hyperlink) = value else {
            return Err(RecordError::NotAHyperlink(path, Box::new(value.to_owned())));
        };

        Ok(Some(hyperlink.to_owned()))
    }

    pub fn string_enum<K: Into<FieldPath>, T: for<'a> TryFrom<&'a str>>(&self, key: K) -> Result<T, RecordError> {
        let path = key.into();
        let Some(value) = self.0.get(&path) else {
            return Err(RecordError::FieldNotPresent(path));
        };
        let FieldValue::String(string) = value else {
            return Err(RecordError::NotAString(path, Box::new(value.to_owned())));
        };

        let Ok(enum_variant) = T::try_from(string.as_str()) else {
            return Err(RecordError::NotAValidEnumVariant(path, string.to_owned()));
        };

        Ok(enum_variant)
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
