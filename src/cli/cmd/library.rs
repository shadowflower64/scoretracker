use scoretracker::config::Config;
use scoretracker::library::database::LibraryDatabase;
use scoretracker::library::{LibraryScanError, scan_full};
use scoretracker::{log_fn_name, success_npr};
use std::path::Path;

pub fn rescan_library(library_dir_path: &Path) -> Result<(), LibraryScanError> {
    log_fn_name!("rescan_library");

    let shared_data_repo_path = Config::load().unwrap().shared_data_repo_path;
    let library_database_path = shared_data_repo_path.join(LibraryDatabase::standard_path());

    scan_full(library_dir_path, &library_database_path, None)?;

    success_npr!("successfully rescanned library");
    Ok(())
}
