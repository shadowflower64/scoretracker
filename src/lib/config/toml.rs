use crate::util::dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TomlConfigError {
    #[error("config file at {path:?} could not be read: {e}")]
    ReadError { path: PathBuf, e: io::Error },
    #[error("config file at {path:?} could not be written to: {e}")]
    WriteError { path: PathBuf, e: io::Error },
    #[error("config file at {path:?} could not be parsed as toml: {e}")]
    DeserializeError { path: PathBuf, e: toml::de::Error },
    #[error("config file could not be serialized as toml: {e}")]
    SerializeError { path: PathBuf, e: toml::ser::Error },
}

pub trait TomlConfig: Sized + Serialize + for<'a> Deserialize<'a> {
    const STANDARD_FILENAME: &str;

    fn default_path() -> PathBuf {
        config_dir().join(Self::STANDARD_FILENAME)
    }

    fn load() -> Result<Self, TomlConfigError> {
        Self::load_from_file(Self::default_path())
    }

    fn load_from_file(path: impl AsRef<Path>) -> Result<Self, TomlConfigError> {
        let file_contents = Self::load_raw_from_file(&path)?;
        toml::from_str(&file_contents).map_err(|e| TomlConfigError::DeserializeError {
            path: path.as_ref().to_path_buf(),
            e,
        })
    }

    fn load_raw_from_file(path: impl AsRef<Path>) -> Result<String, TomlConfigError> {
        fs::read_to_string(&path).map_err(|e| TomlConfigError::ReadError {
            path: path.as_ref().to_path_buf(),
            e,
        })
    }

    fn write_new(&self, path: impl AsRef<Path>) -> Result<(), TomlConfigError> {
        let toml = toml::to_string_pretty(&self).map_err(|e| TomlConfigError::SerializeError {
            path: path.as_ref().to_path_buf(),
            e,
        })?;
        Self::write_new_raw(path, &toml)
    }

    fn write_new_raw(path: impl AsRef<Path>, toml: &str) -> Result<(), TomlConfigError> {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(|e| TomlConfigError::WriteError {
                path: path.as_ref().to_path_buf(),
                e,
            })?;
        file.write_all(toml.as_bytes()).map_err(|e| TomlConfigError::WriteError {
            path: path.as_ref().to_path_buf(),
            e,
        })?;
        Ok(())
    }

    fn write_raw(path: impl AsRef<Path>, toml: &str) -> Result<(), TomlConfigError> {
        let mut file = OpenOptions::new()
            .write(true)
            .open(&path)
            .map_err(|e| TomlConfigError::WriteError {
                path: path.as_ref().to_path_buf(),
                e,
            })?;
        file.write_all(toml.as_bytes()).map_err(|e| TomlConfigError::WriteError {
            path: path.as_ref().to_path_buf(),
            e,
        })?;
        Ok(())
    }
}
