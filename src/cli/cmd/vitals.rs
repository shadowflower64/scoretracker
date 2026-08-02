use crate::cmd::CmdError;
use scoretracker::{
    config::Config,
    data::library::database::LibraryDatabase,
    util::{
        filelocked::{FileLockableData, FileLockableDataWithDefaultPath},
        lockfile,
        terminal_colors::{ANSI_COLOR_BOLD_GREEN, ANSI_COLOR_BOLD_RED, ANSI_COLOR_RESET, ANSI_ERASE_TO_END, ansi_move_cursor_left},
    },
};
use std::{
    fmt,
    io::{Write, stdout},
    path::PathBuf,
    thread::sleep,
    time::Duration,
};
use thiserror::Error;

fn print_check_name(msg: &str) {
    print!("{msg:.<90}");
}

fn print_check_status(status: &str) {
    let movement = ansi_move_cursor_left(status.len() as u32);
    print!("{ANSI_ERASE_TO_END}{status}{movement}");
    stdout().flush().expect("could not flush stdout");
}

fn print_check_ok() {
    println!("{ANSI_ERASE_TO_END}{ANSI_COLOR_BOLD_GREEN}ok{ANSI_COLOR_RESET}");
}

fn print_check_err<E: fmt::Display>(error: &E) {
    println!("{ANSI_ERASE_TO_END}{ANSI_COLOR_BOLD_RED}error: {ANSI_COLOR_RESET}{error}");
}

#[derive(Error, Debug)]
pub enum LibraryDatabaseCheckError {
    #[error("lockfile error: {0}")]
    Lockfile(#[from] lockfile::Error),
}

fn check_library_database(library_db_path: PathBuf) -> Result<(), LibraryDatabaseCheckError> {
    print_check_status("waiting for filelock...");
    let _library_db = LibraryDatabase::lock_and_read(library_db_path, None)?;
    print_check_status("checking...");
    sleep(Duration::from_secs(5)); // TODO
    Ok(())
}

pub fn check_all() -> Result<(), CmdError> {
    let config_path = Config::default_path();
    println!("config located at: {config_path:?}");
    print_check_name(&format!("checking config"));

    let config = match Config::load() {
        Ok(config) => {
            print_check_ok();
            config
        }
        Err(e) => {
            print_check_err(&e);
            return Err(CmdError::ConfigReadError(e));
        }
    };

    let library_db_path = config.library_database_path();
    println!("library database located at: {library_db_path:?}");
    print_check_name(&format!("checking library database"));

    match check_library_database(library_db_path) {
        Ok(_) => {
            print_check_ok();
        }
        Err(e) => {
            print_check_err(&e);
        }
    };

    Ok(())
}
