use function_name::named;

use crate::{
    data::library::database::LibraryEntry, debug, hive::worker::DEBUG_WORKER_TEMP_FILE_CLEANUP, log_fn_name, log_should_print_debug,
    util::uuid::UuidString, warn,
};
use std::path::{Path, PathBuf};

/// Result of fetching a file from a library.
///
/// The ownership of this struct represents ownership of the downloaded or referenced file.
/// Dropping this struct will remove the downloaded file, or do nothing if the file was just referenced.
#[derive(Debug, Clone)]
pub enum Get {
    /// The file is a copy of the original file; the file was downloaded from an external source.
    ///
    /// Modifying this file *will not* modify the original.
    Downloaded {
        library_entry: LibraryEntry,
        downloaded_file: PathBuf,
        delete_on_drop: bool,
    },

    /// The file is just a reference to an existing file on the filesystem. It should be treated as read-only.
    ///
    /// Modifying this file *will* modify the original.
    Referenced {
        library_entry: LibraryEntry,
        referenced_file: PathBuf,
    },
}

impl Get {
    pub fn new_downloaded(library_entry: LibraryEntry, downloaded_file: PathBuf) -> Self {
        Self::Downloaded {
            library_entry,
            downloaded_file,
            delete_on_drop: true, // downloaded files are deleted on drop by default
        }
    }

    pub fn new_referenced(library_entry: LibraryEntry, referenced_file: PathBuf) -> Self {
        Self::Referenced {
            library_entry,
            referenced_file,
        }
    }

    pub fn entry(&self) -> &LibraryEntry {
        match self {
            Self::Downloaded { library_entry, .. } | Self::Referenced { library_entry, .. } => library_entry,
        }
    }

    pub fn uuid(&self) -> UuidString {
        self.entry().uuid
    }

    pub fn read_only_path(&self) -> &Path {
        match self {
            Self::Downloaded { downloaded_file, .. } => downloaded_file,
            Self::Referenced { referenced_file, .. } => referenced_file,
        }
    }

    /// Drop the handle without removing the downloaded file.
    pub fn leave(mut self) {
        match &mut self {
            Self::Downloaded { delete_on_drop, .. } => {
                *delete_on_drop = false;
            }
            _ => {}
        }
        // self drops here
    }

    fn delete_on_drop(&self) -> bool {
        match self {
            Self::Downloaded { delete_on_drop, .. } => *delete_on_drop, // delete downloaded files if the flag is set to true
            Self::Referenced { .. } => false,                           // never delete referenced files
        }
    }
}

impl Drop for Get {
    #[named]
    fn drop(&mut self) {
        log_fn_name!(auto);
        log_should_print_debug!(DEBUG_WORKER_TEMP_FILE_CLEANUP);
        if self.delete_on_drop() {
            debug!("removing downloaded copied file: {self:?}");
            let _ = trash::delete(self.read_only_path()).inspect_err(|e| {
                warn!("could not move downloaded file to trash: {e}");
            });
        }
    }
}
