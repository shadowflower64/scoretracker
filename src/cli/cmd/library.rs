use crate::cmd::Error;
use scoretracker::config::Config;
use scoretracker::library::database::LibraryDatabase;
use scoretracker::library::info::LibraryInfo;
use scoretracker::library::stpl_url::{LibraryDomain, LibraryDomainName};
use scoretracker::library::{LibraryScanError, remove_library_domain_from_db, scan_full};
use scoretracker::util::file_ex::FileEx;
use scoretracker::util::lockfile;
use scoretracker::{log_fn_name, success_npr};
use std::path::Path;

pub fn init(library_dir: &Path, library_domain_name: LibraryDomainName) -> Result<(), Error> {
    log_fn_name!("init");

    let info = LibraryInfo {
        domain: library_domain_name,
    };
    library_dir
        .join(LibraryInfo::STANDARD_FILENAME)
        .write_as_json_pretty(&info)
        .map_err(Error::LibraryInfoWriteError)?;

    success_npr!("initialized library with domain '{}'", LibraryDomain::Local(info.domain));
    Ok(())
}

pub fn rescan(library_dir: &Path) -> Result<(), LibraryScanError> {
    log_fn_name!("rescan");

    let shared_data_repo_path = Config::load().unwrap().shared_data_repo_path;
    let library_db_path = shared_data_repo_path.join(LibraryDatabase::standard_path());

    scan_full(library_dir, &library_db_path, None)?;

    success_npr!("successfully rescanned library");
    Ok(())
}

pub fn remove_domain(library_domain: LibraryDomain) -> Result<(), lockfile::Error> {
    log_fn_name!("remove_domain");

    let shared_data_repo_path = Config::load().unwrap().shared_data_repo_path;
    let library_db_path = shared_data_repo_path.join(LibraryDatabase::standard_path());

    remove_library_domain_from_db(library_domain.clone(), &library_db_path, None)?;

    success_npr!("successfully removed urls with the domain '{library_domain}' from database");
    Ok(())
}
