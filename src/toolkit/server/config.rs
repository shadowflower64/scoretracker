use scoretracker::config::toml::TomlConfig;
use scoretracker::data::library::database::LibraryDatabase;
use scoretracker::data::scoreboard::r#match::MatchDatabase;
use scoretracker::data::scoreboard::performance::PerformanceDatabase;
use scoretracker::data::scoreboard::player::PlayerDatabase;
use scoretracker::hive::queue::TaskQueue;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ServerConfig {
    // pub display_name: String,
    pub shared_data_repo_path: PathBuf,
}

impl TomlConfig for ServerConfig {
    const STANDARD_FILENAME: &str = "server.toml";
}

impl ServerConfig {
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
