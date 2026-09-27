use crate::toolkit::cmd::handle_command;
use scoretracker::{cli::cmdline_context::CmdlineContext, error_npr, util::log};
use std::{env::args, process::ExitCode};

mod toolkit;

#[cfg(feature = "include-server-in-toolkit")]
mod server;
#[cfg(feature = "include-worker-in-toolkit")]
mod worker;

fn main() -> ExitCode {
    let args: Vec<_> = args().collect();
    log::open_default_log_file().expect("could not open log file"); // TODO: open log file only when needed (server, worker, long tasks), don't log to file on short tasks

    let mut cmdline_context = CmdlineContext::new(&args);
    let result = handle_command(&mut cmdline_context);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error_npr!("{}", error);
            error.exit_status()
        }
    }
}
