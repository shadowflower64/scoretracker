use crate::hive::worker::WorkerInfo;
use crate::util::file_ex::{self, FileEx};
use crate::util::lockfile::{self, LockfileHandle};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct FileLocked<T: FileLockableData> {
    pub inner: T,
    lockfile: LockfileHandle,
}

pub trait FileLockableData
where
    Self: Sized + for<'a> serde::Deserialize<'a>,
{
    fn read_without_locking<P: AsRef<Path>>(path: P) -> file_ex::Result<Self> {
        Ok(path.as_ref().read_from_json()?.ok_or(file_ex::Error::file_not_found())?)
    }
    fn lock_and_read<P: AsRef<Path>>(path: P, worker_info: Option<&WorkerInfo>) -> lockfile::Result<FileLocked<Self>> {
        let lockfile = LockfileHandle::acquire_wait(path, worker_info)?;
        let inner = lockfile.read_from_json()?.ok_or(file_ex::Error::file_not_found())?;
        Ok(FileLocked { inner, lockfile })
    }
}

pub trait FileLockableDataWithDefaultPath: FileLockableData {
    fn default_path() -> PathBuf;
    fn read_default_without_locking() -> file_ex::Result<Self> {
        Self::read_without_locking(Self::default_path())
    }
    fn lock_default_and_read(worker_info: Option<&WorkerInfo>) -> lockfile::Result<FileLocked<Self>> {
        Self::lock_and_read(Self::default_path(), worker_info)
    }
}

impl<T: FileLockableData> FileLocked<T> {
    pub fn unlock(self) -> lockfile::Result<()> {
        self.lockfile.unlock()
    }
}

impl<T: FileLockableData + serde::Serialize> FileLocked<T> {
    pub fn write_to_file(&self) -> lockfile::Result<()> {
        Ok(self.lockfile.write_as_json_pretty(&self.inner)?)
    }
}
