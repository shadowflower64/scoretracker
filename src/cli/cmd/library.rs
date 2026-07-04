use scoretracker::config::Config;
use scoretracker::library::database::LibraryDatabase;
use scoretracker::library::stpl_url::LibraryDomain;
use scoretracker::library::{LibraryScanError, remove_library_domain_from_db, scan_full};
use scoretracker::{log_fn_name, success_npr};
use std::path::Path;

pub fn rescan_library(library_dir_path: &Path) -> Result<(), LibraryScanError> {
    log_fn_name!("rescan_library");

    let shared_data_repo_path = Config::load().unwrap().shared_data_repo_path;
    let library_db_path = shared_data_repo_path.join(LibraryDatabase::standard_path());

    scan_full(library_dir_path, &library_db_path, None)?;

    success_npr!("successfully rescanned library");
    Ok(())
}

pub fn remove_domain_from_db(library_domain: LibraryDomain) -> Result<(), LibraryScanError> {
    log_fn_name!("remove_library_from_db");

    let shared_data_repo_path = Config::load().unwrap().shared_data_repo_path;
    let library_db_path = shared_data_repo_path.join(LibraryDatabase::standard_path());

    remove_library_domain_from_db(library_domain.clone(), &library_db_path, None)?;

    success_npr!("successfully removed urls with the domain '{library_domain}' from database");
    Ok(())
}
