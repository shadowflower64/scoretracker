use scoretracker::data::library::stpl_url::LibraryDomain;
use scoretracker::util::dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{fs, io};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerConfigError {
    #[error("config file at {path:?} could not be read: {e}")]
    ReadError { path: PathBuf, e: io::Error },
    #[error("config file at  {path:?} could not be parsed as toml: {e}")]
    TomlError { path: PathBuf, e: toml::de::Error },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InternalLibrary {
    pub paths: Vec<PathBuf>,
    pub domain: LibraryDomain,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub internal_libraries: Vec<InternalLibrary>,
}

impl ServerConfig {
    pub const STANDARD_FILENAME: &str = "server.toml";
    pub fn default_path() -> PathBuf {
        config_dir().join(Self::STANDARD_FILENAME)
    }

    pub fn load() -> Result<Self, ServerConfigError> {
        Self::load_from_file(Self::default_path())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, ServerConfigError> {
        let file_contents = fs::read_to_string(&path).map_err(|e| ServerConfigError::ReadError {
            path: path.as_ref().to_path_buf(),
            e,
        })?;
        toml::from_str(&file_contents).map_err(|e| ServerConfigError::TomlError {
            path: path.as_ref().to_path_buf(),
            e,
        })
    }
}
