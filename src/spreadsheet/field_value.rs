use std::time::Duration;

use calamine::{Data, ExcelDateTime, Hyperlink};
use chrono::{DateTime, NaiveDate, NaiveDateTime};

use crate::{debug, error, log_fn_name, log_should_print_debug, util::timestamp::NsTimestamp, warn};

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

pub fn parse_cell_value(cell: &Data, hyperlink: Option<&Hyperlink>, formula_mode: bool, log_prefix: Option<&str>) -> FieldValue {
    log_fn_name!("parse_cell_value");
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
                return FieldValue::DateTimeWithMsNoTz(datetime);
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
                debug!("{log_prefix}parsing as naive date only, success: {date}");
                return FieldValue::DateOnlyNoTz(date);
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
                debug!(
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
            debug!("{log_prefix}parsing as rfc3339 datetime, success: {datetime}");
            return FieldValue::DateTime(NsTimestamp::from(datetime));
        } else {
            warn!("{log_prefix}parsing as rfc3339 datetime, fail: {result:?}");
        }
        FieldValue::Empty
    };
    let parse_duration = |cell: &str| {
        let result = iso8601_duration::Duration::parse(cell);
        if let Ok(duration) = result {
            debug!("{log_prefix}parsing duration, success: {duration}");
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
