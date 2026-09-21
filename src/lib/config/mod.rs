pub mod library_tab;
pub mod toml;
pub mod toolkit;

use crate::data::library::aux_data::LibraryAuxData;
use crate::data::library::cache::LibraryCache;
use crate::data::library::index::LibraryIndex;
use crate::data::library::stpl_url::LibraryDomain;
use crate::hive::queue::TaskQueue;
use crate::util::dirs::config_dir;
use crate::util::file_ex;
use crate::util::filelocked::{FileLockableDataJson, FileLockableDataWithDefaultPath};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::sync::LazyLock;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LegacyConfig {
    pub shared_data_repo_path: PathBuf,
    pub default_library_dir_path: PathBuf,
    pub default_library: Option<LibraryDomain>,
}

impl LegacyConfig {
    pub fn load() -> Result<&'static Self, &'static file_ex::Error> {
        static LOADED_CONFIG: LazyLock<file_ex::Result<LegacyConfig>> = LazyLock::new(LegacyConfig::read_default_without_locking);
        LOADED_CONFIG.as_ref()
    }

    pub fn task_queue_path(&self) -> PathBuf {
        self.shared_data_repo_path.join(TaskQueue::STANDARD_FILENAME)
    }

    pub fn default_library_index_path(&self) -> PathBuf {
        self.default_library_dir_path.join(LibraryIndex::STANDARD_FILENAME)
    }

    pub fn default_library_cache_path(&self) -> PathBuf {
        self.default_library_dir_path.join(LibraryCache::STANDARD_FILENAME)
    }

    pub fn default_library_aux_data_path(&self) -> PathBuf {
        self.default_library_dir_path.join(LibraryAuxData::STANDARD_FILENAME)
    }
}

impl LegacyConfig {
    pub const STANDARD_FILENAME: &str = "scoretracker_config.json";
    fn default_path_static() -> PathBuf {
        config_dir().join(Self::STANDARD_FILENAME)
    }
}

impl FileLockableDataJson for LegacyConfig {}
impl FileLockableDataWithDefaultPath for LegacyConfig {
    fn default_path() -> PathBuf {
        env::var("SCORETRACKER_CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or(Self::default_path_static())
    }
}
