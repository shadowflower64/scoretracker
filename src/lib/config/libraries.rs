use crate::data::library::stpl_url::LibraryDomain;
use crate::util::dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibraryTableError {
    #[error("library table at {path:?} could not be read: {e}")]
    ReadError { path: PathBuf, e: io::Error },
    #[error("library table at {path:?} could not be written to: {e}")]
    WriteError { path: PathBuf, e: io::Error },
    #[error("library table at {path:?} could not be parsed as toml: {e}")]
    DeserializeError { path: PathBuf, e: toml::de::Error },
    #[error("library table could not be serialized as toml: {e}")]
    SerializeError { path: PathBuf, e: toml::ser::Error },
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LibraryTable {
    pub internal_libraries: HashMap<LibraryDomain, Vec<PathBuf>>,
}

impl LibraryTable {
    pub const STANDARD_FILENAME: &str = "libraries.toml";
    pub fn default_path() -> PathBuf {
        config_dir().join(Self::STANDARD_FILENAME)
    }

    pub fn load() -> Result<Self, LibraryTableError> {
        Self::load_from_file(Self::default_path())
    }

    pub fn load_raw() -> Result<String, LibraryTableError> {
        Self::load_raw_from_file(Self::default_path())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, LibraryTableError> {
        let file_contents = Self::load_raw_from_file(&path)?;
        toml::from_str(&file_contents).map_err(|e| LibraryTableError::DeserializeError {
            path: path.as_ref().to_path_buf(),
            e,
        })
    }

    pub fn load_raw_from_file(path: impl AsRef<Path>) -> Result<String, LibraryTableError> {
        fs::read_to_string(&path).map_err(|e| LibraryTableError::ReadError {
            path: path.as_ref().to_path_buf(),
            e,
        })
    }

    pub fn write_new(&self, path: impl AsRef<Path>) -> Result<(), LibraryTableError> {
        let toml = toml::to_string_pretty(&self).map_err(|e| LibraryTableError::SerializeError {
            path: path.as_ref().to_path_buf(),
            e,
        })?;
        Self::write_new_raw(path, &toml)
    }

    pub fn write_new_raw(path: impl AsRef<Path>, toml: &str) -> Result<(), LibraryTableError> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(|e| LibraryTableError::WriteError {
                path: path.as_ref().to_path_buf(),
                e,
            })?;
        file.write_all(toml.as_bytes()).map_err(|e| LibraryTableError::WriteError {
            path: path.as_ref().to_path_buf(),
            e,
        })?;
        Ok(())
    }

    pub fn write_raw(path: impl AsRef<Path>, toml: &str) -> Result<(), LibraryTableError> {
        let mut file = OpenOptions::new()
            .write(true)
            .open(&path)
            .map_err(|e| LibraryTableError::WriteError {
                path: path.as_ref().to_path_buf(),
                e,
            })?;
        file.write_all(toml.as_bytes()).map_err(|e| LibraryTableError::WriteError {
            path: path.as_ref().to_path_buf(),
            e,
        })?;
        Ok(())
    }
}
