use function_name::named;
use scoretracker::{error, info, log_fn_name, util::log};
use std::process::ExitCode;

use crate::server::http::start::http_server_start;

mod server;

#[named]
fn main() -> ExitCode {
    // let args: Vec<_> = env::args().collect();
    log::open_default_log_file().expect("could not open log file");
    log_fn_name!(auto);

    info!("starting server");
    match http_server_start() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error!("critical server error: {}", error);
            ExitCode::FAILURE
        }
    }
}
