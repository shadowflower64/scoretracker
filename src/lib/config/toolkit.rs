use crate::{config::toml::TomlConfig, data::library::stpl_url::LibraryDomain};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ToolkitConfig {
    pub default_library: Option<LibraryDomain>,
}

impl TomlConfig for ToolkitConfig {
    const STANDARD_FILENAME: &str = "toolkit.toml";
}
