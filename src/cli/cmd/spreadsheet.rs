use crate::cmd::Error;
use calamine::{Data, Ods, Range, Reader, open_workbook};
use chrono::{DateTime, NaiveDate, NaiveDateTime};
use scoretracker::{info, info_npr, log_fn_name, success, util::timestamp::NsTimestamp, warn};
use std::{collections::HashMap, path::Path};

#[derive(Debug)]
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

type Record = HashMap<String, FieldValue>;

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

fn scan_worksheet(sheet_name: &str, range: Range<Data>) -> Vec<Record> {
    log_fn_name!("scan_worksheet");

    info!("parsing sheet: {sheet_name}");

    let mut rows = range.rows();
    let Some(header_row) = rows.next() else {
        return Vec::new();
    };
    let header_row = header_row.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    let mut records = Vec::new();
    for (i, row) in rows.enumerate() {
        let mut record = HashMap::new();
        for (field_name, cell) in header_row.iter().zip(row) {
            // Skip fields that start with `.`
            if field_name.starts_with(".") {
                continue;
            }
            let prefix = format!("sheet: '{sheet_name}', row: {}, field: '{field_name}', value: '{cell}'", i + 2);
            let field_value = parse_cell_value(cell, Some(&prefix));
            record.insert(field_name.to_owned(), field_value);
        }
        records.push(record);
    }
    records
}

pub fn import_legacy(spreadsheet_path: &Path) -> Result<(), Error> {
    let mut workbook: Ods<_> = open_workbook(spreadsheet_path).unwrap();

    let mut worksheets = workbook.worksheets();
    let total_worksheets = worksheets.len();
    worksheets.retain(|(name, _range)| name.starts_with("j."));
    let filtered_worksheets = worksheets.len();

    let names: Vec<_> = worksheets.iter().map(|(name, _range)| name.to_string()).collect();
    info_npr!("total worksheets: {total_worksheets}, filtered worksheets: {filtered_worksheets}, names: {names:?}");

    for (name, range) in worksheets {
        let a = scan_worksheet(&name, range);
        println!("{a:#?}");
    }

    Ok(())
}
