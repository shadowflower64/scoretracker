use crate::error::CmdError;
use scoretracker::util::{dirs::log_dir, reveal_directory};
use scoretracker::{info_npr, success_npr};

pub fn open() -> Result<(), CmdError> {
    let log_dir = log_dir();
    info_npr!("opening log directory: {log_dir:?}");
    reveal_directory(&log_dir).map_err(CmdError::RevealDirectoryError)?;
    success_npr!("successfully opened log directory");
    Ok(())
}
