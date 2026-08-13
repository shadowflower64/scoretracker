use crate::spreadsheet::field_value::{CellContents, parse_cell_contents};
use crate::spreadsheet::{BadRecordError, FieldPath, FieldValue};
use crate::util::timestamp::NsTimestamp;
use crate::{info, log_fn_name};
use calamine::{Data, Hyperlink, Range};
use chrono::NaiveDate;
use chrono_tz::Tz;
use indexmap::IndexMap;
use std::fmt::{self, Display};

#[derive(Default, Clone, Debug)]
pub struct Record(IndexMap<FieldPath, CellContents>);

impl Record {
    fn new() -> Self {
        Default::default()
    }

    /// Returns `Ok(true)` if the field exists and is empty.
    /// Returns `Ok(false)` if the field exists and is not empty.
    /// Returns `Err(_)` if the field does not exist.
    pub fn is_empty<K: Into<FieldPath>>(&self, key: K) -> Result<bool, BadRecordError> {
        self.field_contents(key).map(CellContents::is_empty)
    }

    /// Returns `Ok(true)` if the field exists and is filled.
    /// Returns `Ok(false)` if the field exists and is not filled.
    /// Returns `Err(_)` if the field does not exist.
    pub fn is_filled<K: Into<FieldPath>>(&self, key: K) -> Result<bool, BadRecordError> {
        self.field_contents(key).map(CellContents::is_filled)
    }

    /// Returns `Some(_)` if the field exists.
    pub fn field<K: Into<FieldPath>>(&self, key: K) -> Option<&CellContents> {
        self.0.get(&key.into())
    }

    /// Returns `Ok(_)` if the field exists.
    pub fn field_contents<K: Into<FieldPath>>(&self, key: K) -> Result<&CellContents, BadRecordError> {
        let path = key.into();
        self.0.get(&path).ok_or_else(|| BadRecordError::FieldNotPresent(path))
    }

    /// Returns `Ok(_)` if the field exists and the cell is not empty.
    pub fn field_value<K: Into<FieldPath>>(&self, key: K) -> Result<&FieldValue, BadRecordError> {
        let path = key.into();
        let Some(cell) = self.0.get(&path) else {
            return Err(BadRecordError::FieldNotPresent(path));
        };
        let Some(value) = cell.val() else {
            return Err(BadRecordError::CellIsEmpty(path));
        };
        Ok(value)
    }

    /// Parse the field as a `String`.
    ///
    /// Returns `Ok(String)` if a string is present in the cell.
    /// Returns an `Err(_)` if the field does not exist, if the cell is empty, or if the cell contains another data type.
    pub fn string<K: Into<FieldPath>>(&self, key: K) -> Result<&str, BadRecordError> {
        let path = key.into();
        let value = self.field_value(path.clone())?;
        value
            .as_str()
            .ok_or_else(|| BadRecordError::NotAString(path, Box::new(value.clone())))
    }

    /// Parse the field as an `Option<String>`.
    ///
    /// Returns `Ok(Some(String))` if a string is present in the cell, or `Ok(None)` if the cell is empty.
    /// Returns an `Err(_)` if the field does not exist, or if the cell contains another data type.
    pub fn string_opt<K: Into<FieldPath>>(&self, key: K) -> Result<Option<&str>, BadRecordError> {
        let path = key.into();
        let Some(value) = self.field_contents(path.clone())?.val() else {
            return Ok(None);
        };
        value
            .as_str()
            .ok_or_else(|| BadRecordError::NotAString(path, Box::new(value.clone())))
            .map(Some)
    }

    /// Parse the field as a variable-type field, getting a `String` type.
    ///
    /// Returns `Ok(Some(String))` if a string is present in the cell, or `Ok(None)` if the cell is empty or contains another data type.
    /// Returns an `Err(_)` if the field does not exist.
    pub fn string_var<K: Into<FieldPath>>(&self, key: K) -> Result<Option<&str>, BadRecordError> {
        let path = key.into();
        Ok(self.field_contents(path.clone())?.val().and_then(|x| x.as_str()))
    }

    /// Parse the field as a `i64`.
    ///
    /// Returns `Ok(i64)` if an int is present in the cell.
    /// Returns an `Err(_)` if the field does not exist, if the cell is empty, or if the cell contains another data type.
    pub fn i64<K: Into<FieldPath>>(&self, key: K) -> Result<i64, BadRecordError> {
        let path = key.into();
        let value = self.field_value(path.clone())?;
        value
            .as_i64()
            .ok_or_else(|| BadRecordError::NotAnInt(path, Box::new(value.clone())))
    }

    /// Parse the field as an `Option<i64>`.
    ///
    /// Returns `Ok(Some(i64))` if an int is present in the cell, or `Ok(None)` if the cell is empty.
    /// Returns an `Err(_)` if the field does not exist, or if the cell contains another data type.
    pub fn i64_opt<K: Into<FieldPath>>(&self, key: K) -> Result<Option<i64>, BadRecordError> {
        let path = key.into();
        let Some(value) = self.field_contents(path.clone())?.val() else {
            return Ok(None);
        };
        value
            .as_i64()
            .ok_or_else(|| BadRecordError::NotAnInt(path, Box::new(value.clone())))
            .map(Some)
    }

    /// Parse the field as a variable-type field, getting a `i64` type.
    ///
    /// Returns `Ok(Some(i64))` if an int is present in the cell, or `Ok(None)` if the cell is empty or contains another data type.
    /// Returns an `Err(_)` if the field does not exist.
    pub fn i64_var<K: Into<FieldPath>>(&self, key: K) -> Result<Option<i64>, BadRecordError> {
        let path = key.into();
        Ok(self.field_contents(path.clone())?.val().and_then(|x| x.as_i64()))
    }

    /// Parse the field as something convertible from `i64`.
    ///
    /// Returns `Ok(T)` if an int is present in the cell.
    /// Returns an `Err(_)` if the field does not exist, if the cell is empty, or if the cell contains another data type.
    pub fn int<T: TryFrom<i64>, K: Into<FieldPath>>(&self, key: K) -> Result<T, BadRecordError> {
        let path = key.into();
        let value = self.field_value(path.clone())?;
        value
            .as_int()
            .ok_or_else(|| BadRecordError::NotAnIntSubtype(path, Box::new(value.clone())))
    }

    /// Parse the field as something convertible from `Option<i64>`.
    ///
    /// Returns `Ok(Some(T))` if an int is present in the cell, or `Ok(None)` if the cell is empty.
    /// Returns an `Err(_)` if the field does not exist, or if the cell contains another data type.
    pub fn int_opt<T: TryFrom<i64>, K: Into<FieldPath>>(&self, key: K) -> Result<Option<T>, BadRecordError> {
        let path = key.into();
        let Some(value) = self.field_contents(path.clone())?.val() else {
            return Ok(None);
        };
        value
            .as_int()
            .ok_or_else(|| BadRecordError::NotAnInt(path, Box::new(value.clone())))
            .map(Some)
    }

    /// Parse the field as a variable-type field, getting something convertible from `i64`.
    ///
    /// Returns `Ok(Some(T))` if an int is present in the cell, or `Ok(None)` if the cell is empty or contains another data type.
    /// Returns an `Err(_)` if the field does not exist.
    pub fn int_var<T: TryFrom<i64>, K: Into<FieldPath>>(&self, key: K) -> Result<Option<T>, BadRecordError> {
        let path = key.into();
        Ok(self.field_contents(path.clone())?.val().and_then(|x| x.as_int()))
    }

    /// Parse the field as a `f64`.
    ///
    /// Returns `Ok(f64)` if a float is present in the cell.
    /// Returns an `Err(_)` if the field does not exist, if the cell is empty, or if the cell contains another data type.
    pub fn f64<K: Into<FieldPath>>(&self, key: K) -> Result<f64, BadRecordError> {
        let path = key.into();
        let value = self.field_value(path.clone())?;
        value
            .as_f64()
            .ok_or_else(|| BadRecordError::NotAFloat(path, Box::new(value.clone())))
    }

    /// Parse the field as something convertible from `f64`.
    ///
    /// Returns `Ok(T)` if a float is present in the cell.
    /// Returns an `Err(_)` if the field does not exist, if the cell is empty, or if the cell contains another data type.
    pub fn float<T: TryFrom<f64>, K: Into<FieldPath>>(&self, key: K) -> Result<T, BadRecordError> {
        let path = key.into();
        let value = self.field_value(path.clone())?;
        value
            .as_float()
            .ok_or_else(|| BadRecordError::NotAFloatSubtype(path, Box::new(value.clone())))
    }

    /// Parse the field as a `bool`.
    ///
    /// Returns `Ok(bool)` if a boolean value is present in the cell.
    /// Returns an `Err(_)` if the field does not exist, if the cell is empty, or if the cell contains another data type.
    pub fn bool<K: Into<FieldPath>>(&self, key: K) -> Result<bool, BadRecordError> {
        let path = key.into();
        let value = self.field_value(path.clone())?;
        value
            .as_bool()
            .ok_or_else(|| BadRecordError::NotABool(path, Box::new(value.clone())))
    }

    /// Parse the field as an `Option<bool>`.
    ///
    /// Returns `Ok(bool)` if a boolean value is present in the cell.
    /// Returns an `Err(_)` if the field does not exist, if the cell is empty, or if the cell contains another data type.
    pub fn bool_opt<K: Into<FieldPath>>(&self, key: K) -> Result<Option<bool>, BadRecordError> {
        let path = key.into();
        let Some(value) = self.field_contents(path.clone())?.val() else {
            return Ok(None);
        };
        value
            .as_bool()
            .ok_or_else(|| BadRecordError::NotABool(path, Box::new(value.clone())))
            .map(Some)
    }

    /// Parse the field as a timestamp.
    ///
    /// Returns `Ok(NsTimestamp)` if a timestamp is present in the cell.
    /// Returns an `Err(_)` if the field does not exist, if the cell is empty, or if the cell contains another data type.
    pub fn timestamp<K: Into<FieldPath>>(&self, key: K, tz: Tz) -> Result<NsTimestamp, BadRecordError> {
        let path = key.into();
        let value = self.field_value(path.clone())?;
        value.try_as_timestamp(path, tz)
    }

    /// Parse the field as a `NaiveDate`.
    ///
    /// Returns `Ok(NaiveDate)` if a date is present in the cell.
    /// Returns an `Err(_)` if the field does not exist, if the cell is empty, or if the cell contains another data type.
    pub fn date_only<K: Into<FieldPath>>(&self, key: K) -> Result<NaiveDate, BadRecordError> {
        let path = key.into();
        let value = self.field_value(path.clone())?;
        value
            .as_date()
            .ok_or_else(|| BadRecordError::NotADate(path, Box::new(value.clone())))
    }

    /// Parse the field as a `Hyperlink`.
    ///
    /// Returns `Ok(Hyperlink)` if a hyperlink is present in the cell.
    /// Returns an `Err(_)` if the field does not exist, if the cell is empty, or if the cell contains another data type.
    pub fn hyperlink<K: Into<FieldPath>>(&self, key: K) -> Result<&Hyperlink, BadRecordError> {
        let path = key.into();
        let value = self.field_value(path.clone())?;
        value
            .as_hyperlink()
            .ok_or_else(|| BadRecordError::NotAHyperlink(path, Box::new(value.clone())))
    }

    /// Parse the field as an `Option<Hyperlink>`.
    ///
    /// Returns Ok(Hyperlink) if a hyperlink is present in the cell.
    /// Returns an `Err(_)` if the field does not exist, if the cell is empty, or if the cell contains another data type.
    pub fn hyperlink_opt<K: Into<FieldPath>>(&self, key: K) -> Result<Option<&Hyperlink>, BadRecordError> {
        let path = key.into();
        let Some(value) = self.field_contents(path.clone())?.val() else {
            return Ok(None);
        };
        value
            .as_hyperlink()
            .ok_or_else(|| BadRecordError::NotAnInt(path, Box::new(value.clone())))
            .map(Some)
    }

    /// Parse the field as a variable-type field, getting a `Hyperlink` type.
    ///
    /// Returns `Ok(Some(Hyperlink))` if a hyperlink is present in the cell, or `Ok(None)` if the cell is empty or contains another data type.
    /// Returns an `Err(_)` if the field does not exist.
    pub fn hyperlink_var<K: Into<FieldPath>>(&self, key: K) -> Result<Option<&Hyperlink>, BadRecordError> {
        let path = key.into();
        Ok(self.field_contents(path.clone())?.val().and_then(|x| x.as_hyperlink()))
    }

    pub fn string_enum<K: Into<FieldPath>, T: for<'a> TryFrom<&'a str, Error = &'static str>>(&self, key: K) -> Result<T, BadRecordError> {
        let path = key.into();
        let string = self.string(path.clone())?;
        T::try_from(string).map_err(|enum_name| BadRecordError::NotAValidEnumVariant(path, enum_name.to_string(), string.to_owned()))
    }
}

impl AsRef<IndexMap<FieldPath, CellContents>> for Record {
    fn as_ref(&self) -> &IndexMap<FieldPath, CellContents> {
        &self.0
    }
}
impl AsMut<IndexMap<FieldPath, CellContents>> for Record {
    fn as_mut(&mut self) -> &mut IndexMap<FieldPath, CellContents> {
        &mut self.0
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            let field_value = parse_cell_contents(cell, hyperlink, formula_mode, Some(&prefix));
            record.as_mut().insert(field_name.into(), field_value);
        }
        records.push(record);
    }
    records
}
