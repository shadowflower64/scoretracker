use crate::{
    config::toml::{TomlConfig, TomlConfigError},
    data::library::stpl_url::LibraryDomain,
};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ToolkitConfig {
    pub default_library: Option<LibraryDomain>,
    pub database_connection: Option<String>,
    pub database_schema: Option<String>,
}

impl ToolkitConfig {
    pub fn global() -> Result<&'static Self, &'static TomlConfigError> {
        static LOADED_CONFIG: LazyLock<Result<ToolkitConfig, TomlConfigError>> = LazyLock::new(ToolkitConfig::load);
        LOADED_CONFIG.as_ref()
    }
}

impl TomlConfig for ToolkitConfig {
    const STANDARD_FILENAME: &str = "toolkit.toml";
}
