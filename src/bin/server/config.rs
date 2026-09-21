use scoretracker::config::toml::TomlConfig;
use scoretracker::hive::queue::TaskQueue;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ServerConfig {
    // pub display_name: String,
    pub shared_data_repo_path: PathBuf,
    pub database_connection: String,
    pub database_schema: String,
}

impl TomlConfig for ServerConfig {
    const STANDARD_FILENAME: &str = "server.toml";
}

impl ServerConfig {
    pub fn task_queue_path(&self) -> PathBuf {
        self.shared_data_repo_path.join(TaskQueue::STANDARD_FILENAME)
    }
}
