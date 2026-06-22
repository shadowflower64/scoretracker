use serde::{Deserialize, Serialize};

use crate::hive::worker::WorkerInfo;
use crate::util::file_ex::{self, FileEx};
use crate::util::lockfile::{self, LockfileHandle};
use std::fs;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct FileLocked<T: FileLockableData> {
    pub inner: T,
    lockfile: LockfileHandle,
}

pub trait FileLockableData: Sized {
    fn _inner_read<F: FileEx + ?Sized>(file_ex: &F) -> file_ex::Result<Option<Self>>;
    fn _inner_write<F: FileEx + ?Sized>(&self, file_ex: &F) -> file_ex::Result<()>;

    fn read_without_locking<P: AsRef<Path>>(path: P) -> file_ex::Result<Self> {
        Self::_inner_read(path.as_ref())?.ok_or(file_ex::Error::file_not_found())
    }
    fn lock_and_read<P: AsRef<Path>>(path: P, worker_info: Option<&WorkerInfo>) -> lockfile::Result<FileLocked<Self>> {
        let lockfile = LockfileHandle::acquire_wait(path, worker_info)?;
        let inner = Self::_inner_read(&lockfile)?.ok_or(file_ex::Error::file_not_found())?;
        Ok(FileLocked { inner, lockfile })
    }
}

pub trait FileLockableDataJson {}
impl<T> FileLockableData for T
where
    T: for<'a> Deserialize<'a> + Serialize + FileLockableDataJson,
{
    fn _inner_read<F: FileEx + ?Sized>(file_ex: &F) -> file_ex::Result<Option<Self>> {
        file_ex.read_from_json()
    }

    fn _inner_write<F: FileEx + ?Sized>(&self, file_ex: &F) -> file_ex::Result<()> {
        file_ex.write_as_json_pretty(self)
    }
}

pub trait FileLockableDataDefault: FileLockableData + Default {
    fn read_without_locking_or_default<P: AsRef<Path>>(path: P) -> file_ex::Result<Self> {
        Ok(Self::_inner_read(path.as_ref())?.unwrap_or_default())
    }

    /// Loads data from a file or creates a new data structure.
    ///
    /// This function loads the data from a file at the provided file path, or creates a new empty data structure if the file does not exist.
    ///
    /// This function will return Err when:
    /// * the file could not be read to string, or
    /// * the file structure could not be parsed.
    ///
    /// This is to prevent overwriting existing data if it has become corrupted or protected by permissions.
    fn lock_and_read_or_default<P: AsRef<Path>>(path: P, worker_info: Option<&WorkerInfo>) -> lockfile::Result<FileLocked<Self>> {
        let lockfile = LockfileHandle::acquire_wait(path, worker_info)?;
        let inner = Self::_inner_read(&lockfile)?.unwrap_or_default();
        Ok(FileLocked { inner, lockfile })
    }
}

impl<T> FileLockableDataDefault for T where T: FileLockableData + Default {}

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
    pub fn close(self) -> ClosedFileLocked<T> {
        ClosedFileLocked {
            main_file_path: self.lockfile.main_file_path().to_path_buf(),
            phantom: PhantomData,
        }
    }
}

impl<T: FileLockableData> FileLocked<T> {
    pub fn write_to_file(&self) -> lockfile::Result<()> {
        Ok(T::_inner_write(&self.inner, &self.lockfile)?)
    }
    pub fn create_parent_dirs_and_write_to_file(&self) -> lockfile::Result<()> {
        let _ = self
            .lockfile
            .main_file_path()
            .parent()
            .and_then(|parent| fs::create_dir_all(parent).ok());
        Ok(T::_inner_write(&self.inner, &self.lockfile)?)
    }
}

#[derive(Debug)]
pub struct ClosedFileLocked<T> {
    main_file_path: PathBuf,
    phantom: PhantomData<T>,
}

impl<T: FileLockableData> Deref for FileLocked<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: FileLockableData> DerefMut for FileLocked<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: FileLockableData> ClosedFileLocked<T> {
    pub fn reopen(self, worker_info: Option<&WorkerInfo>) -> lockfile::Result<FileLocked<T>> {
        T::lock_and_read(self.main_file_path, worker_info)
    }
}
