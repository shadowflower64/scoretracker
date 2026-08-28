use serde::{Deserialize, Serialize};

use crate::hive::worker::data::WorkerInfo;
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
    worker_info: Option<WorkerInfo>,
}

pub trait FileLockableData: Sized {
    fn _inner_read<F: FileEx + ?Sized>(file_ex: &F) -> file_ex::Result<Option<Self>>;
    fn _inner_write<F: FileEx + ?Sized>(&self, file_ex: &F) -> file_ex::Result<()>;

    fn read_without_locking(path: impl AsRef<Path>) -> file_ex::Result<Self> {
        Self::_inner_read(path.as_ref())?.ok_or_else(|| file_ex::Error::file_not_found(path.as_ref().to_path_buf()))
    }
    fn lock_and_read(path: impl AsRef<Path>, worker_info: Option<&WorkerInfo>) -> lockfile::Result<FileLocked<Self>> {
        let lockfile = LockfileHandle::acquire_wait(&path, worker_info)?;
        let inner = Self::_inner_read(&lockfile)?.ok_or_else(|| file_ex::Error::file_not_found(path.as_ref().to_path_buf()))?;
        Ok(FileLocked {
            inner,
            lockfile,
            worker_info: worker_info.cloned(),
        })
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
    fn read_without_locking_or_default(path: impl AsRef<Path>) -> file_ex::Result<Self> {
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
    fn lock_and_read_or_default(path: impl AsRef<Path>, worker_info: Option<&WorkerInfo>) -> lockfile::Result<FileLocked<Self>> {
        let lockfile = LockfileHandle::acquire_wait(path, worker_info)?;
        let inner = Self::_inner_read(&lockfile)?.unwrap_or_default();
        Ok(FileLocked {
            inner,
            lockfile,
            worker_info: worker_info.cloned(),
        })
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
    pub fn unlock_without_saving(self) -> lockfile::Result<()> {
        self.lockfile.unlock()
    }
    pub fn close_without_saving(self) -> lockfile::Result<ClosedFileLocked<T>> {
        let a = ClosedFileLocked {
            main_file_path: self.lockfile.main_file_path().to_path_buf(),
            worker_info: self.worker_info,
            phantom: PhantomData,
        };
        self.lockfile.unlock()?;
        Ok(a)
    }
}

impl<T: FileLockableData> FileLocked<T> {
    pub fn save_to_file(&self) -> lockfile::Result<()> {
        Ok(T::_inner_write(&self.inner, &self.lockfile)?)
    }

    pub fn save_and_unlock(self) -> lockfile::Result<()> {
        self.save_to_file()?;
        self.unlock_without_saving()
    }

    pub fn save_and_close(self) -> lockfile::Result<ClosedFileLocked<T>> {
        self.save_to_file()?;
        self.close_without_saving()
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
    worker_info: Option<WorkerInfo>,
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
    pub fn reopen(self) -> lockfile::Result<FileLocked<T>> {
        T::lock_and_read(self.main_file_path, self.worker_info.as_ref())
    }

    pub fn reopen_with_new_worker_info(self, worker_info: Option<&WorkerInfo>) -> lockfile::Result<FileLocked<T>> {
        T::lock_and_read(self.main_file_path, worker_info)
    }
}

#[derive(Debug)]
enum Clopen<T: FileLockableData> {
    Open(FileLocked<T>),
    Closed(ClosedFileLocked<T>),
}

#[derive(Debug)]
pub struct ClosedOrOpen<T: FileLockableData> {
    // Invariant: this is always Some, it is only None temporarily while closing/opening.
    some: Option<Clopen<T>>,
}

impl<T: FileLockableData> From<FileLocked<T>> for ClosedOrOpen<T> {
    fn from(value: FileLocked<T>) -> Self {
        ClosedOrOpen {
            some: Some(Clopen::Open(value)),
        }
    }
}

impl<T: FileLockableData> From<ClosedFileLocked<T>> for ClosedOrOpen<T> {
    fn from(value: ClosedFileLocked<T>) -> Self {
        ClosedOrOpen {
            some: Some(Clopen::Closed(value)),
        }
    }
}

impl<T: FileLockableData> ClosedOrOpen<T> {
    pub fn open(&mut self) -> Result<&mut FileLocked<T>, lockfile::Error> {
        let some = self.some.take().unwrap();
        match some {
            Clopen::Open(open) => {
                // Put it back gently, its already open...
                self.some = Some(Clopen::Open(open));
            }
            Clopen::Closed(closed) => {
                // Put it back in the opened state
                let reopened = closed.reopen()?;
                self.some = Some(Clopen::Open(reopened));
            }
        }

        match self.some.as_mut().unwrap() {
            Clopen::Open(open) => Ok(open),
            _ => unreachable!(),
        }
    }

    pub fn save_and_close(&mut self) -> Result<&mut ClosedFileLocked<T>, lockfile::Error> {
        let some = self.some.take().unwrap();
        match some {
            Clopen::Closed(closed) => {
                // Put it back gently, its already closed...
                self.some = Some(Clopen::Closed(closed));
            }
            Clopen::Open(open) => {
                // Put it back in the closed state
                let reopened = open.save_and_close()?;
                self.some = Some(Clopen::Closed(reopened));
            }
        }

        match self.some.as_mut().unwrap() {
            Clopen::Closed(closed) => Ok(closed),
            _ => unreachable!(),
        }
    }
}
