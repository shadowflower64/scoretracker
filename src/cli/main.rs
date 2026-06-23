use crate::util::handle_command;
use scoretracker::error_npr;
use std::{env::args, process::ExitCode};

pub mod cmd;
pub mod util;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let args: Vec<_> = args().collect();

    let result = handle_command(&args);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error_npr!("{}", error);
            error.exit_status()
        }
    }
}
