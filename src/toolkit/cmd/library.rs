use scoretracker::config::Config;
use scoretracker::library::{database::LibraryDatabaseLock, index::LibraryIndex};
use scoretracker::util::{file_ex, lockfile};
use scoretracker::{info, log_fn_name, success_npr};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibraryScanError {
    #[error("could not read library database: {0}")]
    LibraryDatabaseReadError(lockfile::Error),
    #[error("could not write to library database: {0}")]
    LibraryDatabaseWriteError(lockfile::Error),
    #[error("could not write to library index: {0}")]
    LibraryIndexWriteError(file_ex::Error),
}

// todo this should be moved to core
pub fn rescan(library_dir_path: &Path) -> Result<(), LibraryScanError> {
    log_fn_name!("rescan_library");

    let shared_data_repo_path = Config::load().unwrap().shared_data_repo_path;

    let library_index_path = library_dir_path.join(LibraryIndex::STANDARD_FILENAME);
    let library_database_path = shared_data_repo_path.join(LibraryDatabaseLock::STANDARD_FILENAME);

    let mut library_database =
        LibraryDatabaseLock::read_or_create_new_safe(&library_database_path, None).map_err(LibraryScanError::LibraryDatabaseReadError)?;
    let library_index = LibraryIndex::scan_library_dir(library_dir_path, &mut library_database);

    library_index
        .save(&library_index_path)
        .map_err(LibraryScanError::LibraryIndexWriteError)?;
    info!("saved library index to {library_index_path:?}");

    library_database
        .write_to_file()
        .map_err(LibraryScanError::LibraryDatabaseWriteError)?;
    info!("saved library data to {library_database_path:?}");

    success_npr!("successfully rescanned library");
    Ok(())
}
