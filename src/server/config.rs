use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub internal_library_dirs: Vec<PathBuf>,
}

impl ServerConfig {
    pub fn load() -> Self {
        todo!();
    }
}
