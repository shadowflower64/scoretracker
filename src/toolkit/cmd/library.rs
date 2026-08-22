use crate::cmd::CmdError;
use function_name::named;
use scoretracker::config::Config;
use scoretracker::config::libraries::{LibraryTable, LibraryTableError};
use scoretracker::data::library::info::LibraryInfo;
use scoretracker::data::library::stpl_url::LibraryDomain;
use scoretracker::data::library::{remove_library_domain_from_db, scan_full};
use scoretracker::util::file_ex::FileEx;
use scoretracker::util::filelocked::FileLockableData;
use scoretracker::{log_fn_name, success_npr};
use std::borrow::Cow;
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;
use toml_edit::DocumentMut;

#[derive(Debug, Clone)]
pub enum LibraryIdentifier {
    DomainName(LibraryDomain),
    DirPath(PathBuf),
}

#[derive(Debug, Error)]
pub enum LibraryIdentifierError {
    #[error("cannot load library table: {0}")]
    CannotLoadLibraryTable(LibraryTableError),
    #[error("domain not found")]
    DomainNotFound,
    #[error("domain has no paths")]
    DomainHasNoPaths,
}

impl LibraryIdentifier {
    pub fn detect_from_pathbuf(path_or_domain: PathBuf) -> Self {
        let string = path_or_domain.to_string_lossy().to_string();
        if let Ok(domain) = LibraryDomain::try_from(string) {
            LibraryIdentifier::DomainName(domain)
        } else {
            LibraryIdentifier::DirPath(path_or_domain)
        }
    }

    pub fn detect_from_string(path_or_domain: String) -> Self {
        if let Ok(domain) = LibraryDomain::try_from(path_or_domain.clone()) {
            LibraryIdentifier::DomainName(domain)
        } else {
            LibraryIdentifier::DirPath(PathBuf::from(path_or_domain))
        }
    }

    pub fn dir_path(&self) -> Result<Cow<'_, Path>, LibraryIdentifierError> {
        type E = LibraryIdentifierError;
        match self {
            LibraryIdentifier::DomainName(domain) => Ok(Cow::Owned(
                LibraryTable::load() // TODO: load this only once, like a config file
                    .map_err(E::CannotLoadLibraryTable)?
                    .internal_libraries
                    .get(domain)
                    .ok_or(E::DomainNotFound)?
                    .first()
                    .ok_or(E::DomainHasNoPaths)?
                    .to_owned(),
            )),
            LibraryIdentifier::DirPath(path) => Ok(Cow::Borrowed(path)),
        }
    }
}

impl FromStr for LibraryIdentifier {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::detect_from_string(s.to_string()))
    }
}

#[named]
pub fn init(library_dir: &Path, domain: LibraryDomain) -> Result<(), CmdError> {
    log_fn_name!(auto);

    let info = LibraryInfo { domain };
    library_dir
        .join(LibraryInfo::STANDARD_FILENAME)
        .write_as_json_pretty(&info)
        .map_err(CmdError::LibraryInfoWriteError)?;

    success_npr!("initialized library with domain '{}'", info.domain);
    Ok(())
}

#[named]
pub fn install(library_dir: &Path) -> Result<(), CmdError> {
    log_fn_name!(auto);

    let info =
        LibraryInfo::read_without_locking(library_dir.join(LibraryInfo::STANDARD_FILENAME)).map_err(CmdError::LibraryInfoReadError)?;

    let source_toml = LibraryTable::load_raw()?;
    let mut document: DocumentMut = source_toml.parse().expect("todo: invalid library table");

    let internal_libraries_tab = document["internal_libraries"].as_table_mut().expect("todo: invalid library table");
    if let Some(existing) = internal_libraries_tab.get_mut(info.domain.as_ref()) {
        let paths_arr = existing.as_array_mut().expect("todo: invalid library table");
        let exists = paths_arr
            .iter()
            .find(|x| x.as_str().expect("todo: invalid library table") == library_dir)
            .is_some();
        if exists {
            panic!("todo: path is already installed");
        } else {
            paths_arr.push(library_dir.to_string_lossy().to_string());
        }
    } else {
        let mut paths_arr = toml_edit::Array::new();
        paths_arr.push(library_dir.to_string_lossy().to_string());
        internal_libraries_tab.insert(info.domain.as_ref(), paths_arr.into());
    }

    let modified_toml = document.to_string();
    LibraryTable::write_raw(LibraryTable::default_path(), &modified_toml).expect("todo: write error");

    success_npr!("installed library with domain '{}'", info.domain);
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
