use crate::util::dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{fs, io};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerConfigError {
    #[error("config file could not be read: {0}")]
    ReadError(io::Error),
    #[error("config file could not be parsed as toml: {0}")]
    TomlError(toml::de::Error),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub internal_library_dirs: Vec<PathBuf>,
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
        let file_contents = fs::read_to_string(path).map_err(ServerConfigError::ReadError)?;
        toml::from_str(&file_contents).map_err(ServerConfigError::TomlError)
    }
}
