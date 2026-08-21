use scoretracker::data::library::database::LibraryDatabase;
use scoretracker::data::scoreboard::r#match::MatchDatabase;
use scoretracker::data::scoreboard::performance::PerformanceDatabase;
use scoretracker::data::scoreboard::player::PlayerDatabase;
use scoretracker::hive::queue::TaskQueue;
use scoretracker::util::dirs::config_dir;
use serde::{Deserialize, Serialize};
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
    // pub display_name: String,
    pub shared_data_repo_path: PathBuf,
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

    pub fn library_database_path(&self) -> PathBuf {
        LibraryDatabase::path_within_shared_repo().to_path(&self.shared_data_repo_path)
    }

    pub fn match_database_path(&self) -> PathBuf {
        MatchDatabase::path_within_shared_repo().to_path(&self.shared_data_repo_path)
    }

    pub fn performance_database_path(&self) -> PathBuf {
        PerformanceDatabase::path_within_shared_repo().to_path(&self.shared_data_repo_path)
    }

    pub fn player_database_path(&self) -> PathBuf {
        PlayerDatabase::path_within_shared_repo().to_path(&self.shared_data_repo_path)
    }

    pub fn task_queue_path(&self) -> PathBuf {
        self.shared_data_repo_path.join(TaskQueue::STANDARD_FILENAME)
    }
}
