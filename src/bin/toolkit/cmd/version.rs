use crate::toolkit::error::CmdError;
use scoretracker::SCORETRACKER_VERSION;

pub fn version() -> Result<(), CmdError> {
    println!("scoretracker v{SCORETRACKER_VERSION}");
    Ok(())
}
