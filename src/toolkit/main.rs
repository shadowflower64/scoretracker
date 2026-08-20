use crate::cmd::handle_command;
use scoretracker::{error_npr, util::log};
use std::{env::args, process::ExitCode};

pub mod arg;
pub mod cmd;
pub mod error;
pub mod server;

fn main() -> ExitCode {
    let args: Vec<_> = args().collect();
    log::open_default_log_file().expect("could not open log file");

    let result = handle_command(&args);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error_npr!("{}", error);
            error.exit_status()
        }
    }
}
