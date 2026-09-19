//! `secrets.toml` config file handling.
//!
//! This module handles reading the `secrets.toml` global config file, which contains database credentials.
//! This config is used by the server to connect to the database.

use crate::config::toml::{TomlConfig, TomlConfigError};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SecretsConfig {
    pub database_connection_params: Option<String>,
}

impl TomlConfig for SecretsConfig {
    const STANDARD_FILENAME: &str = "secrets.toml";
}

impl SecretsConfig {
    pub fn get() -> Result<&'static Self, &'static TomlConfigError> {
        static LOADED_CONFIG: LazyLock<Result<SecretsConfig, TomlConfigError>> = LazyLock::new(SecretsConfig::load);
        LOADED_CONFIG.as_ref()
    }
}

// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub struct DatabaseConfig {
//     pub host: String,
//     pub port: u16,
//     pub username: String,
//     pub password: String,
// }
