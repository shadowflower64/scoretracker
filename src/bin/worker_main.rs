use function_name::named;
use scoretracker::{error, info, log_fn_name, util::log};
use std::process::ExitCode;

use crate::worker::start::worker_start;

mod worker;

#[named]
fn main() -> ExitCode {
    // let args: Vec<_> = env::args().collect();
    log::open_default_log_file().expect("could not open log file");
    log_fn_name!(auto);

    info!("starting worker");
    match worker_start() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error!("critical worker error: {}", error);
            ExitCode::FAILURE
        }
    }
}
