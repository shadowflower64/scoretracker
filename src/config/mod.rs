use crate::hive::queue::TaskQueue;
use crate::hive::worker::WorkerInfo;
use crate::library::aux_data::LibraryAuxData;
use crate::library::cache::LibraryCache;
use crate::library::database::LibraryDatabase;
use crate::library::index::LibraryIndex;
use crate::util::dirs::config_dir;
use crate::util::filelocked::{FileLockableDataJson, FileLockableDataWithDefaultPath};
use crate::util::lockfile::{self};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub domain_name: String,
    pub shared_data_repo_path: PathBuf,
    pub default_library_dir_path: PathBuf,
}

impl Config {
    pub fn load() -> lockfile::Result<Self> {
        Self::load_with_worker(None)
    }

    pub fn load_with_worker(_worker_info: Option<&WorkerInfo>) -> lockfile::Result<Self> {
        // TODO: decide on whether this should actually be locking or not
        Ok(Config::read_default_without_locking()?)
    }

    pub fn library_database_path(&self) -> PathBuf {
        self.shared_data_repo_path.join(LibraryDatabase::standard_path())
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

impl Config {
    pub const STANDARD_FILENAME: &str = "scoretracker_config.json";
    fn default_path_static() -> PathBuf {
        config_dir().join(Self::STANDARD_FILENAME)
    }
}

impl FileLockableDataJson for Config {}
impl FileLockableDataWithDefaultPath for Config {
    fn default_path() -> PathBuf {
        env::var("SCORETRACKER_CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or(Self::default_path_static())
    }
}
