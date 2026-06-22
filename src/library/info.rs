//! Library info file handling.
//!
//! A "library info file" is a file that contains basic information about the library.
//! For example, this file contains the domain name for this library dir.
use crate::hive::worker::WorkerInfo;
use crate::util::file_ex::FileEx;
use crate::util::lockfile::{self, LockfileHandle};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Basic info about the library.
///
/// This structure contains information about the library. Currently, it only contains the domain name.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LibraryInfo {
    pub domain: String,
}

/// Wrapper for handling library info files. See [`LibraryInfo`] for more documentation.
#[derive(Debug)]
pub struct LibraryInfoLock {
    inner: LibraryInfo,
    lockfile: LockfileHandle,
}

impl LibraryInfoLock {
    pub const STANDARD_FILENAME: &str = "library_info.json";

    pub fn read_or_create_new_safe<P: AsRef<Path>>(path: P, worker_info: Option<&WorkerInfo>) -> lockfile::Result<Self> {
        let lockfile = LockfileHandle::acquire_wait(path, worker_info)?;
        let inner = lockfile.read_from_json()?.unwrap_or_default();
        Ok(Self { inner, lockfile })
    }

    pub fn write_to_file(&self) -> lockfile::Result<()> {
        Ok(self.lockfile.write_as_json_pretty(&self.inner)?)
    }
}
