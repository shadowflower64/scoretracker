use crate::toolkit::cmd::CmdError;
use function_name::named;
use rand::{RngExt, distr};
use relative_path::RelativePathBuf;
use scoretracker::cli::cmdline_argument::CmdlineArgument;
use scoretracker::config::library_tab::LibraryTab;
use scoretracker::config::toml::{TomlConfig, TomlConfigError};
use scoretracker::data::library::stpl_url::LibraryDomain;
use scoretracker::db::{Database, DbError};
use scoretracker::library::info::LibraryInfo;
use scoretracker::library::library_scan_full;
use scoretracker::{log_fn_name, success_npr, warn_npr};
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
    CannotLoadLibraryTable(TomlConfigError),
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
                LibraryTab::read_default() // TODO: load this only once, like a config file
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

impl CmdlineArgument for LibraryIdentifier {
    fn arg_type() -> &'static str {
        "library identifier"
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

    let info = LibraryInfo {
        domain,
        authenticity_string: rand::rng().sample_iter(&distr::Alphanumeric).take(32).map(char::from).collect(),
        temp_dir: Some(RelativePathBuf::from("temp")),
    };
    let path = library_dir.join(LibraryInfo::STANDARD_FILENAME);
    info.write_new(path).map_err(CmdError::LibraryInfoWriteError)?;

    success_npr!("initialized library with domain '{}'", info.domain);
    Ok(())
}

#[named]
pub fn install(library_dir: &Path) -> Result<(), CmdError> {
    log_fn_name!(auto);

    let info = LibraryInfo::read(library_dir.join(LibraryInfo::STANDARD_FILENAME)).map_err(CmdError::LibraryInfoReadError)?;

    let source_toml = LibraryTab::read_raw_default().map_err(CmdError::LibraryTableError)?;
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
    println!("{}", modified_toml);
    LibraryTab::write_raw(LibraryTab::default_path(), &modified_toml).expect("todo: write error");

    success_npr!("installed library with domain '{}' located at: {:?}", info.domain, library_dir);
    Ok(())
}

#[named]
pub fn uninstall(library_dir: &Path) -> Result<(), CmdError> {
    log_fn_name!(auto);

    let mut found_domains = Vec::new();

    let source_toml = LibraryTab::read_raw_default().map_err(CmdError::LibraryTableError)?;
    let mut document: DocumentMut = source_toml.parse().expect("todo: invalid library table");

    let internal_libraries_tab = document["internal_libraries"].as_table_mut().expect("todo: invalid library table");
    for (key, item) in internal_libraries_tab.iter_mut() {
        let domain = key.to_string();

        let paths_arr = item.as_array_mut().expect("todo: invalid library table");
        paths_arr.retain(|value| {
            if let Some(path_string) = value.as_str()
                && path_string == library_dir
            {
                found_domains.push(domain.clone());
                // delete this element
                false
            } else {
                // keep all other elemments
                true
            }
        });
    }

    let modified_toml = document.to_string();
    println!("{}", modified_toml);
    LibraryTab::write_raw(LibraryTab::default_path(), &modified_toml).expect("todo: write error");

    if found_domains.is_empty() {
        warn_npr!("could not uninstall library located at: {library_dir:?} - it was not installed");
    } else {
        success_npr!("uninstalled library with domains: {found_domains:?} located at: {library_dir:?} ");
    }

    Ok(())
}

#[named]
pub fn list() -> Result<(), CmdError> {
    log_fn_name!(auto);

    let tab = LibraryTab::read_default().map_err(CmdError::LibraryTableError)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&tab).map_err(CmdError::ConfigSerializationError)?
    );
    Ok(())
}

#[named]
pub fn scan(library_dir: &Path) -> Result<(), CmdError> {
    log_fn_name!(auto);

    smol::block_on(async {
        library_scan_full(library_dir).await?;
        Result::<(), CmdError>::Ok(())
    })?;

    success_npr!("successfully rescanned library");
    Ok(())
}

#[named]
pub fn remove_domain(library_domain: LibraryDomain) -> Result<(), CmdError> {
    log_fn_name!(auto);

    smol::block_on(async {
        let mut db = Database::connect_with_tokio_for_toolkit().await?;
        db.remove_library_domain_from_db(&library_domain).await?;
        Result::<(), DbError>::Ok(())
    })?;

    success_npr!("successfully removed urls with the domain '{library_domain}' from database");
    Ok(())
}
