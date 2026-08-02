use crate::cmd::CmdError;
use scoretracker::{spreadsheet, success_npr};
use std::path::Path;

pub fn import_org_ods(ods_path: &Path) -> Result<(), CmdError> {
    spreadsheet::import_org_spreadsheet_ods(ods_path)?;
    success_npr!("successfully imported data from {ods_path:?}");
    Ok(())
}

pub fn import_org_xlsx(xlsx_path: &Path) -> Result<(), CmdError> {
    spreadsheet::import_org_spreadsheet_xlsx(xlsx_path)?;
    success_npr!("successfully imported data from {xlsx_path:?}");
    Ok(())
}
