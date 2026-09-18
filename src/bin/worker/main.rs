use function_name::named;
use scoretracker::{error, info, log_fn_name, util::log};
use std::process::ExitCode;

mod start;

#[named]
fn main() -> ExitCode {
    // let args: Vec<_> = env::args().collect();
    log::open_default_log_file().expect("could not open log file");
    log_fn_name!(auto);

    info!("starting worker");
    match start::worker_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error!("critical worker error: {}", error);
            ExitCode::FAILURE
        }
    }
}
