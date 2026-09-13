use std::path::{Path, PathBuf};

use function_name::named;

use crate::{
    data::library::database::LibraryEntry, debug, hive::worker::DEBUG_WORKER_TEMP_FILE_CLEANUP, log_fn_name, log_should_print_debug,
    util::uuid::UuidString, warn,
};

/// Result of uploading a file to a library.
///
/// The ownership of this struct represents ownership of the uploaded or moved file.
/// Dropping this struct will remove the uploaded file if it was copied, or do nothing if the file was just moved.
#[derive(Debug, Clone)]
pub enum Put {
    /// The file was copied or uploaded to an external sink, and the original file was not removed.
    ///
    /// Modifying this file *will not* modify the upload.
    Uploaded {
        library_entry: LibraryEntry,
        original_path: PathBuf,
        delete_on_drop: bool,
    },

    /// The original file was moved without copying, and the original path is no longer present.
    ///
    /// Modifying this file *will* modify the upload.
    Moved {
        library_entry: LibraryEntry,
        original_path: PathBuf,
        destination_path: PathBuf,
    },
}

impl Put {
    pub fn new_uploaded(library_entry: LibraryEntry, original_path: PathBuf) -> Self {
        Self::Uploaded {
            library_entry,
            original_path,
            delete_on_drop: true, // uploaded files are deleted on drop by default
        }
    }

    pub fn new_moved(library_entry: LibraryEntry, original_path: PathBuf, destination_path: PathBuf) -> Self {
        Self::Moved {
            library_entry,
            original_path,
            destination_path,
        }
    }

    pub fn entry(&self) -> &LibraryEntry {
        match self {
            Self::Uploaded { library_entry, .. } | Self::Moved { library_entry, .. } => library_entry,
        }
    }

    pub fn uuid(&self) -> UuidString {
        self.entry().uuid
    }

    /// Returns the path that still points to a valid file that can be read.
    pub fn valid_path(&self) -> &Path {
        match self {
            Self::Uploaded { original_path, .. } => original_path,
            Self::Moved { destination_path, .. } => destination_path,
        }
    }

    /// Drop the handle without removing the downloaded file.
    pub fn leave(mut self) {
        match &mut self {
            Self::Uploaded {
                delete_on_drop: delete, ..
            } => {
                *delete = false;
            }
            _ => {}
        }
        // self drops here
    }

    fn should_be_deleted(&self) -> bool {
        match self {
            Self::Uploaded {
                delete_on_drop: delete, ..
            } => *delete, // delete uploaded files if the flag is set to true
            Self::Moved { .. } => false, // never delete moved files
        }
    }
}

impl Drop for Put {
    #[named]
    fn drop(&mut self) {
        log_fn_name!(auto);
        log_should_print_debug!(DEBUG_WORKER_TEMP_FILE_CLEANUP);
        if self.should_be_deleted() {
            debug!("removing uploaded copied file: {self:?}");
            let _ = trash::delete(self.valid_path()).inspect_err(|e| {
                warn!("could not move uploaded file to trash: {e}");
            });
        }
    }
}
