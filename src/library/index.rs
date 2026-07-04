//! Library index file handling.
//!
//! A library index is a file that maps every filename existing in the library directory to a "proof UUID", which should be shared across all connected libraries.
//!
//! This file is re-created every time the library gets re-scanned for new content.
use crate::util::file_ex::{Error, FileEx};
use crate::util::filelocked::FileLockableDataJson;
use crate::util::uuid::UuidString;
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

/// A mapping from paths to proof UUIDs.
///
/// The library index is a data structure that links specific proof files on disk to proof UUIDs.
///
/// The index does not actually store any meaningful data, and is used only as a quick-access list of all available proof files in the library.
/// It is safe to delete, as it can be reconstructed from the files on disk.
///
/// # Usage
///
/// Here's an example use case of the index:
/// Let's say the user wants to list all available files in the proof library and view the associated scores.
/// Without the index file, the software would have to:
/// 1. Recursively search through the entire library.
/// 2. Calculate the SHA256 hashes of found files (or fetch them from [`crate::library::cache::LibraryCache`]).
/// 3. Open the proof database.
/// 4. Search through all proofs and filter for the ones with matching SHA256 hashes.
/// 5. Note the UUIDs of the filtered proofs.
/// 6. Open the performance database.
/// 7. Search through all scores that reference the proof UUIDs found before.
///
/// With the index file, finding the proof's UUID is as easy as a hashmap lookup. With an up-to-date index file, the process looks like this:
/// 1. Open the index file.
/// 2. Retrieve all proof UUIDs and paths from the index directly.
/// 3. Open the performance database.
/// 4. Search through all scores that reference the proof UUIDs found before.
///
/// Searching through the entire directory recurisvely is moved to the scanning process which can be launched separately,
/// and will only need to be ran once new files are added to the library. See below for details.
///
/// # Scanning
///
/// This data structure goes out of date whenever a new proof file gets added to the library,
/// whenever a proof file gets moved around the library, and whenever a proof is removed from the library.
/// To sync up the data structure again, the library needs to be *rescanned*.
/// Scanning can be done via the [`LibraryIndex::scan_library_dir`] function, which returns
/// an entirely new index structure, which can then be saved to disk.
/// The saved file is usually called [`LibraryIndex::STANDARD_FILENAME`].
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LibraryIndex {
    // Map of 'relative file path' : 'proof UUID'
    pub files: HashMap<RelativePathBuf, UuidString>,
}

impl LibraryIndex {
    pub const STANDARD_FILENAME: &str = "library_index.json";

    pub fn save(&self, path: &Path) -> Result<(), Error> {
        path.write_as_json_pretty(self)?;
        Ok(())
    }
}

impl FileLockableDataJson for LibraryIndex {}
