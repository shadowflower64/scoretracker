use crate::cmd::CmdError;
use function_name::named;
use scoretracker::config::Config;
use scoretracker::data::library::info::LibraryInfo;
use scoretracker::data::library::stpl_url::{LibraryDomain, LibraryDomainName};
use scoretracker::data::library::{remove_library_domain_from_db, scan_full};
use scoretracker::util::file_ex::FileEx;
use scoretracker::{log_fn_name, success_npr};
use std::path::Path;

#[named]
pub fn init(library_dir: &Path, library_domain_name: LibraryDomainName) -> Result<(), CmdError> {
    log_fn_name!(auto);

    let info = LibraryInfo {
        domain: library_domain_name,
    };
    library_dir
        .join(LibraryInfo::STANDARD_FILENAME)
        .write_as_json_pretty(&info)
        .map_err(CmdError::LibraryInfoWriteError)?;

    success_npr!("initialized library with domain '{}'", LibraryDomain::Local(info.domain));
    Ok(())
}

#[named]
pub fn rescan(library_dir: &Path) -> Result<(), CmdError> {
    log_fn_name!(auto);

    let library_db_path = Config::load().map_err(CmdError::ConfigReadError)?.library_database_path();
    scan_full(library_dir, &library_db_path, None)?;

    success_npr!("successfully rescanned library");
    Ok(())
}

#[named]
pub fn remove_domain(library_domain: LibraryDomain) -> Result<(), CmdError> {
    log_fn_name!(auto);

    let library_db_path = Config::load().map_err(CmdError::ConfigReadError)?.library_database_path();
    remove_library_domain_from_db(library_domain.clone(), &library_db_path, None)?;

    success_npr!("successfully removed urls with the domain '{library_domain}' from database");
    Ok(())
}
