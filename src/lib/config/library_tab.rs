//! `library_tab.toml` config file handling.
//!
//! This module handles reading the `library_tab.toml` global config file, which contains a list of all locally available proof libraries.
//! It also contains code for checking the availability of the specified library paths, and provides a structure for storing list of paths and the availability of them.
use crate::config::toml::TomlConfig;
use crate::data::library::info::LibraryInfo;
use crate::data::library::stpl_url::LibraryDomain;
use crate::util::filelocked::FileLockableData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LibraryTab {
    pub internal_libraries: HashMap<LibraryDomain, Vec<PathBuf>>,
}

impl TomlConfig for LibraryTab {
    const STANDARD_FILENAME: &str = "library_tab.toml";
}

impl LibraryTab {
    pub fn scan(self) -> InternalLibraryConnections {
        InternalLibraryConnections::scan_domains(self.internal_libraries)
    }
}

#[derive(Debug)]
pub enum BadLibraryError {
    WrongDomain { expected: LibraryDomain, actual: LibraryDomain },
    FileExError { library_info_path: PathBuf, error_message: String },
}

#[derive(Debug, Clone)]
pub enum Status {
    Available,
    Bad(Arc<BadLibraryError>),
    Unavailable,
}

impl Status {
    pub fn sort_key(&self) -> u8 {
        match self {
            Self::Available => 0,
            Self::Bad(_) => 1,
            Self::Unavailable => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LibraryAccessPath {
    /// Lowest number = biggest priority/most often used.
    pub priority: i32,

    /// Access path to the library.
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct LibraryAccessPathWithStatus {
    /// Main part of the structure.
    body: LibraryAccessPath,

    /// Whether this path is currently available.
    status: Status,
}

impl LibraryAccessPathWithStatus {
    /// Returns `Some(LibraryAccessPath)` if the status of this path entry is [`Status::Available`]. Returns `None` if the path is unavailable.
    pub fn available_path(&self) -> Option<&LibraryAccessPath> {
        match self.status {
            Status::Available => Some(&self.body),
            Status::Bad(_) => None,
            Status::Unavailable => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InternalLibraryConnection {
    /// Invariant: this vec is always sorted so that the accessible path is always first.
    /// You can get the most useful path in O(1) time by using [`Vec::first`] (and checking whether it's available).
    access_paths: Vec<LibraryAccessPathWithStatus>,
}

impl InternalLibraryConnection {
    /// Create a connection from a set of paths with their status.
    pub fn new(mut access_paths: Vec<LibraryAccessPathWithStatus>) -> Self {
        access_paths.sort_by_key(|x| x.body.priority);
        access_paths.sort_by_key(|x| x.status.sort_key());
        Self { access_paths }
    }

    /// Create a connection from a given set of paths to the library.
    ///
    /// This function will also check for the existence of a `library_info.json` file at the provided paths.
    /// If the file cannot be read, the access path is marked as unavailable.
    ///
    /// The path may be checked again using the `redetect` function.
    pub fn scan_paths(paths: Vec<PathBuf>, expected_domain: &LibraryDomain) -> Self {
        let mut paths_with_status = Vec::with_capacity(paths.len());
        let mut priority = 0;
        for path in paths {
            let status = Self::check_path_availability(&path, expected_domain);
            paths_with_status.push(LibraryAccessPathWithStatus {
                body: LibraryAccessPath { priority, path },
                status,
            });
            priority += 1;
        }
        Self::new(paths_with_status)
    }

    /// Return the first path to the library that is actually available for use.
    pub fn main_path(&self) -> Option<&LibraryAccessPath> {
        self.access_paths.first().and_then(|x| x.available_path())
    }

    fn check_path_availability(path: &Path, expected_domain: &LibraryDomain) -> Status {
        let library_info_path = path.join(LibraryInfo::STANDARD_FILENAME);
        if !library_info_path.is_file() {
            return Status::Unavailable;
        }
        match LibraryInfo::read_without_locking(&library_info_path) {
            Ok(library_info) => {
                if &library_info.domain != expected_domain {
                    return Status::Bad(Arc::new(BadLibraryError::WrongDomain {
                        expected: expected_domain.clone(),
                        actual: library_info.domain,
                    }));
                }

                return Status::Available;
            }
            Err(error) => {
                return Status::Bad(Arc::new(BadLibraryError::FileExError {
                    library_info_path,
                    error_message: error.to_string(),
                }));
            }
        }
    }

    /// Refresh all paths' availability status.
    pub fn redetect(&mut self, expected_domain: &LibraryDomain) {
        for access_path in &mut self.access_paths {
            let fresh_status = Self::check_path_availability(&access_path.body.path, expected_domain);
            access_path.status = fresh_status;
        }
    }

    /// Refresh a specific path's availability status.
    ///
    /// Returns `Some(status)` if the path exists within the connection and has been redetected.
    /// Returns `None` if the path doesn't exist within the connection and the function did nothing.
    pub fn redetect_path(&mut self, requested_path: &Path, expected_domain: &LibraryDomain) -> Option<Status> {
        let mut fresh_status = None;
        for access_path in &mut self.access_paths {
            if access_path.body.path == requested_path {
                access_path.status = fresh_status
                    .get_or_insert_with(|| Self::check_path_availability(&requested_path, expected_domain))
                    .clone();
            }
        }
        fresh_status
    }
}

#[derive(Debug)]
pub struct InternalLibraryConnections {
    connections: HashMap<LibraryDomain, InternalLibraryConnection>,
}

impl InternalLibraryConnections {
    pub fn scan_domains(map: HashMap<LibraryDomain, Vec<PathBuf>>) -> Self {
        let mut connections = HashMap::with_capacity(map.len());
        for (domain, paths) in map {
            let connection = InternalLibraryConnection::scan_paths(paths, &domain);
            connections.insert(domain, connection);
        }
        InternalLibraryConnections { connections }
    }

    pub fn get_main_path(&self, domain: &LibraryDomain) -> Option<&LibraryAccessPath> {
        self.connections.get(domain).and_then(|x| x.main_path())
    }

    /// Refresh all paths' availability status.
    pub fn redetect(&mut self) {
        for (domain, connection) in &mut self.connections {
            connection.redetect(domain);
        }
    }
    /// Refresh a specific domain's paths' availability status.
    ///
    /// Returns `true` if the domain actually exists in the connections table.
    /// Returns `false` if it does not, and the function did nothing.
    pub fn redetect_domain(&mut self, domain: &LibraryDomain) -> bool {
        if let Some(connection) = self.connections.get_mut(domain) {
            connection.redetect(domain);
            true
        } else {
            false
        }
    }
}

impl Deref for InternalLibraryConnections {
    type Target = HashMap<LibraryDomain, InternalLibraryConnection>;
    fn deref(&self) -> &Self::Target {
        &self.connections
    }
}
