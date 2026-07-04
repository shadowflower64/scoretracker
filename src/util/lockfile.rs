//! File lock handling.
use crate::hive::worker::WorkerInfo;
use crate::util::file_ex::{self, FileEx};
use crate::util::lockfile::{self};
use crate::util::timestamp::NsTimestamp;
use crate::{VERSION, debug, info, log_fn_name, log_should_print_debug, warn};
use notify::{ErrorKind, Event, RecursiveMode, Watcher};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::result;
use std::sync::mpsc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("no parent path for path: {0}")]
    NoParentPath(PathBuf),
    #[error("no filename for path: {0}")]
    NoFilename(PathBuf),
    #[error("filename is not valid UTF-8: {0:?}")]
    FilenameIsNotUTF8(OsString),
    #[error("cannot create lockfile: {0}")]
    CannotCreateLockfile(io::Error),
    #[error("cannot write to lockfile: {0}")]
    CannotWriteLockfile(io::Error),
    #[error("cannot remove lockfile: {0}")]
    CannotRemoveLockfile(io::Error),
    #[error("cannot get recommended file watcher: {0}")]
    CannotGetRecommendedWatcher(notify::Error),
    #[error("cannot watch lockfile: {0}")]
    CannotWatchLockfile(notify::Error),
    #[error("{0}")]
    FileExError(#[from] file_ex::Error),
}

impl Error {
    pub fn is_already_locked(&self) -> bool {
        match self {
            Self::CannotCreateLockfile(io_error) => io_error.kind() == io::ErrorKind::AlreadyExists,
            _ => false,
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

pub fn is_file_locked<T>(result: &Result<T>) -> bool {
    if let Err(error) = result {
        error.is_already_locked()
    } else {
        false
    }
}

/// A lockfile handle.
///
/// A lockfile is like a warning to other programs and processes, that says - if you write to this file, your changes will most likely be lost.
/// If every process involved uses the same lockfile system, then the system guarantees that only one process at a time has access to the files.
///
/// The lockfile is based on the provided path, with a `.lockfile` suffix appended to it.
/// For example, for a file called `./directory/subdir/test.txt`, the path `./directory/subdir/test.txt.lockfile`.
///
/// Here is how this safety guard process works:
/// 1. The lockfile is created. If the same file existed already, the opening process fails and exits out early.
///    This has to be an atomic process - it is important that the check for the file's existence is NOT separate from the creation of the file.
///    Checking first and then creating the file would create a TOCTOU race condition issue.
///    Thankfully, we can use Rust's [`File::create_new`] function to achieve atomicity.
/// 2. The main file is read, using usual [`std::fs`] methods.
///    We can do this safely, knowing that other processes won't read the file for writing at the same time, for as long as this process' lockfile exists.
/// 3. The read data is processed or modified.
/// 4. The processed data is written to the file.
///    Again - since the lockfile still exists, this guarantees that other processes using this system do not overwrite the file.
/// 5. After writing is finished, the lockfile is removed. This frees up any other processes to take over the file again.
// DEV NOTE: DO NOT ADD CLONE - Owning an instance of this structure represents unique ownership of the associated file. It should never be cloned.
#[derive(Debug)]
pub struct LockfileHandle {
    main_file_path: PathBuf,
    lockfile_path: PathBuf,
    unlocked_manually: bool,
}

impl LockfileHandle {
    const VERBOSE: bool = true;

    fn generate_lockfile_contents(worker_info: Option<&WorkerInfo>) -> String {
        let timestamp = NsTimestamp::now();
        let timestamp_num = timestamp.as_nanos();
        let timestamp_string = timestamp.to_date_time_string_local();
        let pid = process::id();
        let worker_info_string = worker_info
            .map(|worker| {
                let name = &worker.name;
                let timestamp_num = worker.birth_timestamp.as_nanos();
                let timestamp_string = worker.birth_timestamp.to_date_time_string_local();
                let address = worker.address;
                format!(
                    "
worker_name: {name}
worker_birth_timestamp: {timestamp_num}
# worker_birth_timestamp: {timestamp_string}
worker_address: {address}",
                )
            })
            .unwrap_or_default();

        format!(
            "# File locked by scoretracker.
# WARNING - Do not edit the locked file. Editing the locked file may result in data loss.

version: {VERSION}
pid: {pid}
lock_timestamp: {timestamp_num}
# lock_timestamp: {timestamp_string}
{worker_info_string}
",
        )
    }

    fn create_lockfile_on_disk(lockfile_path: &Path, worker_info: Option<&WorkerInfo>) -> Result<()> {
        log_fn_name!("lockfile:create_lockfile_on_disk");
        log_should_print_debug!(LockfileHandle::VERBOSE);

        let parent = lockfile_path.parent().ok_or(Error::NoParentPath(lockfile_path.to_owned()))?;
        let _ = fs::create_dir_all(parent).inspect_err(|e| warn!("could not create parent directories for lockfile: {e}"));
        let mut lockfile = File::create_new(lockfile_path).map_err(Error::CannotCreateLockfile)?;
        lockfile
            .write_all(Self::generate_lockfile_contents(worker_info).as_bytes())
            .map_err(Error::CannotWriteLockfile)?;

        debug!("created lockfile: {:?}", lockfile_path);
        Ok(())
    }

    pub fn lockfile_path_for<P: AsRef<Path>>(path: P) -> Result<PathBuf> {
        let path = path.as_ref();
        let parent = path.parent().ok_or(Error::NoParentPath(path.to_owned()))?;
        let filename_osstr = path.file_name().ok_or(Error::NoFilename(path.to_owned()))?;
        let filename = filename_osstr.to_str().ok_or(Error::FilenameIsNotUTF8(filename_osstr.to_owned()))?;

        let lockfile_path = parent.join(format!("{filename}.lock"));
        Ok(lockfile_path)
    }

    pub fn main_file_path(&self) -> &Path {
        &self.main_file_path
    }

    pub fn lockfile_path(&self) -> &Path {
        &self.lockfile_path
    }

    /// Safely open a file to update it.
    ///
    /// This function first tries to create a lockfile for the given path, and gives you a [`LockfileHandle`] upon success.
    /// If another process is currently working on this file, this function fails, as it cannot create a lockfile.
    ///
    /// The lockfile gets deleted from disk when the returned [`LockfileHandle`] is dropped.
    /// If the lockfile could not be deleted from disk upon dropping, an error message is printed on stderr.
    ///
    /// This doesn't actually open/read the underlying file on its own, only the lockfile.
    ///
    /// This structure also does not hold any file handles in it. It just calls the [`std::fs`] functions when necessary.
    ///
    /// # Errors
    /// If the path for the lockfile cannot be generated, this function may return [`Error::NoParentPath`], [`Error::NoFilename`], or [`Error::FilenameIsNotUTF8`].
    /// If the lockfile could not be created, this function will return [`Error::CannotCreateLockfile`].
    /// If the lockfile could not be written to, this function will return [`Error::CannotWriteLockfile`].
    pub fn acquire<P: AsRef<Path>>(path: P, worker_info: Option<&WorkerInfo>) -> Result<LockfileHandle> {
        // Create lockfile
        let lockfile_path = Self::lockfile_path_for(&path)?;
        Self::create_lockfile_on_disk(&lockfile_path, worker_info)?;

        Ok(Self {
            main_file_path: path.as_ref().to_path_buf(),
            lockfile_path,
            unlocked_manually: false,
        })
    }

    /// Wait until the file can be safely opened to be updated.
    ///
    /// This function first tries to create a lockfile for the given path, and gives you a [`LockfileHandle`] upon success.
    /// If another process is currently working on this file, this function cannot create a lockfile,
    /// and it waits until it detects that the lockfile has been deleted (using the `notify` crate).
    /// After file removal detection, the attempt to create the lockfile is made again.
    /// This process is repeated until this process successfully acquires the lock.
    /// This function exits only when the lockfile was successfully created or if a different error has been encountered.
    ///
    /// The lockfile gets deleted from disk when the returned [`LockfileHandle`] is dropped.
    /// If the lockfile could not be deleted from disk upon dropping, an error message is printed on stderr.
    ///
    /// This doesn't actually open/read the underlying file on its own, only the lockfile.
    ///
    /// This structure also does not hold any file handles in it. It just calls the [`std::fs`] functions when necessary.
    ///
    /// # Errors
    /// If the path for the lockfile cannot be generated, this function may return [`Error::NoParentPath`], [`Error::NoFilename`], or [`Error::FilenameIsNotUTF8`].
    /// If the lockfile could not be written to, this function will return [`Error::CannotWriteLockfile`].
    pub fn acquire_wait<P: AsRef<Path>>(path: P, worker_info: Option<&WorkerInfo>) -> Result<LockfileHandle> {
        log_fn_name!("lockfile:acquire_wait");
        log_should_print_debug!(LockfileHandle::VERBOSE);

        // Try to create initial lockfile
        let initial_result = Self::acquire(&path, worker_info);
        if !is_file_locked(&initial_result) {
            // The file was not locked before - return the initial result, whatever it was.
            debug!("file was unlocked initially");
            return initial_result;
        }

        // Otherwise, the lockfile couldn't be created because it is already locked, wait for the file to be deleted
        // Setup a watcher to watch for file deletion.
        let lockfile_path = Self::lockfile_path_for(&path)?;
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let mut watcher = notify::recommended_watcher(tx).map_err(Error::CannotGetRecommendedWatcher)?;
        watcher
            .watch(&lockfile_path, RecursiveMode::NonRecursive)
            .map_err(Error::CannotWatchLockfile)?;
        // TODO: this ^ will sometimes exit with an error if the file doesn't exist anymore as you can't watch paths that don't exist.
        // the intended behavior for this case is to create the lockfile. this should be very rare though.

        // Theoretically, the file could've been deleted while everything was being set up - check again for the file again
        let result = Self::acquire(&path, worker_info);
        if !is_file_locked(&result) {
            // The file is not locked anymore! - return the result, whatever it was.
            debug!("file has been unlocked during setup");
            return result;
        }

        info!("waiting for file {lockfile_path:?} to be unlocked");
        for res in rx {
            let event = res.unwrap();
            debug!("event about lockfile: {event:?}");

            if event.kind.is_remove() || event.kind.is_modify() {
                // No matter what the event is, we can try to aquire the lockfile again as it might be freed up now.
                // Sometimes the lock is freed even after modification events (such as renaming the file).
                // This may still fail, as another process might've acquired the lock as well.
                let result = Self::acquire(&path, worker_info);
                if !is_file_locked(&result) {
                    // The file is not locked anymore - return the result, whatever it was.
                    debug!("file unlocked from notification");
                    return result;
                }
                // Otherwise, the lockfile is locked still, try again later.
            }

            // At this point, we recieved an event about this file, but we still couldn't lock it.
            // If we couldn't lock it because another process re-created the file, or moved the file somewhere else,
            // then `notify` will no longer notify us about that new file, as it is different from the original file.
            // This most likely means that we need to re-watch it.

            // We can remove the old path and add it again
            let _ = watcher.unwatch(&lockfile_path).inspect_err(|error| {
                if matches!(error.kind, ErrorKind::WatchNotFound) {
                    // This is fine and expected, ignore entirely
                } else {
                    warn!("couldn't unwatch lockfile at {:?}: {error}", &lockfile_path);
                }
            });

            debug!("rewatching lockfile");
            watcher
                .watch(&lockfile_path, RecursiveMode::NonRecursive)
                .map_err(Error::CannotWatchLockfile)?;
            // TODO: this ^ will sometimes exit with an error if the file doesn't exist anymore as you can't watch paths that don't exist.
            // the intended behavior for this case is to create the lockfile. this should be very rare though.
        }

        unreachable!();
    }

    pub fn unlock(mut self) -> lockfile::Result<()> {
        log_fn_name!("lockfile:unlock");
        log_should_print_debug!(LockfileHandle::VERBOSE);

        fs::remove_file(&self.lockfile_path).map_err(Error::CannotRemoveLockfile)?;
        self.unlocked_manually = true;

        debug!("unlocked manually: {:?}", &self.lockfile_path);
        Ok(())
    }
}

impl Drop for LockfileHandle {
    fn drop(&mut self) {
        log_fn_name!("lockfile:drop");
        log_should_print_debug!(LockfileHandle::VERBOSE);

        if self.unlocked_manually {
            // File was already deleted, do not try to delete it again.
            // If another process has created a new file in-between calling [`LockfileHandle::unlock`] and dropping,
            // attempting to delete the lock file can lead to bugs.
            return;
        }

        if let Err(e) = fs::remove_file(&self.lockfile_path) {
            warn!("could not remove lockfile at {:?}: {e:?}", &self.lockfile_path);
        } else {
            debug!("unlocked by dropping: {:?}", &self.lockfile_path);
        }
    }
}

impl FileEx for LockfileHandle {
    fn file_path(&self) -> &Path {
        &self.main_file_path
    }
}
