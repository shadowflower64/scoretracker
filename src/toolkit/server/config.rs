use scoretracker::data::library::stpl_url::LibraryDomain;
use scoretracker::util::dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerConfigError {
    #[error("config file at {path:?} could not be read: {e}")]
    ReadError { path: PathBuf, e: io::Error },
    #[error("config file at {path:?} could not be written to: {e}")]
    WriteError { path: PathBuf, e: io::Error },
    #[error("config file at {path:?} could not be parsed as toml: {e}")]
    DeserializeError { path: PathBuf, e: toml::de::Error },
    #[error("config file could not be serialized as toml: {e}")]
    SerializeError { path: PathBuf, e: toml::ser::Error },
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ServerConfig {
    pub internal_libraries: HashMap<LibraryDomain, Vec<PathBuf>>,
}

impl ServerConfig {
    pub const STANDARD_FILENAME: &str = "server.toml";
    pub fn default_path() -> PathBuf {
        config_dir().join(Self::STANDARD_FILENAME)
    }

    pub fn load() -> Result<Self, ServerConfigError> {
        Self::load_from_file(Self::default_path())
    }

    pub fn load_raw() -> Result<String, ServerConfigError> {
        Self::load_raw_from_file(Self::default_path())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, ServerConfigError> {
        let file_contents = Self::load_raw_from_file(&path)?;
        toml::from_str(&file_contents).map_err(|e| ServerConfigError::DeserializeError {
            path: path.as_ref().to_path_buf(),
            e,
        })
    }

    pub fn load_raw_from_file(path: impl AsRef<Path>) -> Result<String, ServerConfigError> {
        fs::read_to_string(&path).map_err(|e| ServerConfigError::ReadError {
            path: path.as_ref().to_path_buf(),
            e,
        })
    }

    pub fn write_new(&self, path: impl AsRef<Path>) -> Result<(), ServerConfigError> {
        let toml = toml::to_string_pretty(&self).map_err(|e| ServerConfigError::SerializeError {
            path: path.as_ref().to_path_buf(),
            e,
        })?;
        Self::write_new_raw(path, &toml)
    }

    pub fn write_new_raw(path: impl AsRef<Path>, toml: &str) -> Result<(), ServerConfigError> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(|e| ServerConfigError::WriteError {
                path: path.as_ref().to_path_buf(),
                e,
            })?;
        file.write_all(toml.as_bytes()).map_err(|e| ServerConfigError::WriteError {
            path: path.as_ref().to_path_buf(),
            e,
        })?;
        Ok(())
    }

    pub fn write_raw(path: impl AsRef<Path>, toml: &str) -> Result<(), ServerConfigError> {
        let mut file = OpenOptions::new()
            .write(true)
            .open(&path)
            .map_err(|e| ServerConfigError::WriteError {
                path: path.as_ref().to_path_buf(),
                e,
            })?;
        file.write_all(toml.as_bytes()).map_err(|e| ServerConfigError::WriteError {
            path: path.as_ref().to_path_buf(),
            e,
        })?;
        Ok(())
    }
}
