use scoretracker::hive::queue::TaskQueue;
use scoretracker::{config::toml::TomlConfig, db::schema_name::SafeSchemaName};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    // pub display_name: String,
    pub shared_data_repo_path: PathBuf,
    pub database_connection: String,
    pub database_schema: SafeSchemaName,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            shared_data_repo_path: Default::default(),
            database_connection: Default::default(),
            database_schema: "scoretracker_dev".parse().unwrap(),
        }
    }
}

impl TomlConfig for ServerConfig {
    const STANDARD_FILENAME: &str = "server.toml";
}

impl ServerConfig {
    pub fn task_queue_path(&self) -> PathBuf {
        self.shared_data_repo_path.join(TaskQueue::STANDARD_FILENAME)
    }
}
