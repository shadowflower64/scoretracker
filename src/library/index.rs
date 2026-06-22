//! Library index file handling.
//!
//! A library index is a file that maps every filename existing in the library directory to a "proof UUID", which should be shared across all connected libraries.
//!
//! This file is re-created every time the library gets re-scanned for new content.
use crate::library::cache::LibraryCache;
use crate::library::database::LibraryDatabase;
use crate::library::info::LibraryInfo;
use crate::util::file_ex::{Error, FileEx};
use crate::util::filelocked::FileLockableDataDefault;
use crate::util::uuid::UuidString;
use crate::{debug, info, log_fn_name, log_should_print_debug};
use serde::Serialize;
use std::path::PathBuf;
use std::time::Instant;
use std::{collections::HashMap, path::Path};
use walkdir::WalkDir;

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
#[derive(Debug, Clone, Serialize, Default)]
pub struct LibraryIndex {
    // Map of 'relative file path' : 'proof UUID'
    pub files: HashMap<PathBuf, UuidString>,
}

impl LibraryIndex {
    pub const VERBOSE_SCANNING: bool = true;
    pub const STANDARD_FILENAME: &str = "library_index.json";

    pub fn should_file_be_scanned(filename: &str) -> bool {
        filename.ends_with(".mp4") || filename.ends_with(".mkv")
    }

    pub fn scan_library_dir(library_dir: &Path, library_db: &mut LibraryDatabase) -> Self {
        log_fn_name!("scan:scan_library_dir");
        log_should_print_debug!(LibraryIndex::VERBOSE_SCANNING);

        let scanning_start_timestamp = Instant::now();

        let mut index = Self::default();
        let mut cache = LibraryCache::lock_and_read_or_default(library_dir.join(LibraryCache::STANDARD_FILENAME), None)
            .expect("could not read library cache");
        let info = LibraryInfo::read_without_locking_or_default(library_dir.join(LibraryInfo::STANDARD_FILENAME))
            .expect("could not read library info");

        let files_to_scan: Vec<_> = WalkDir::new(library_dir)
            .into_iter()
            .filter_map(|result| {
                result
                    .ok()
                    .and_then(|dir_entry| if dir_entry.file_type().is_file() { Some(dir_entry) } else { None })
            })
            .collect();
        let len = files_to_scan.len();
        let mut skipped = 0;

        for (i, dir_entry) in files_to_scan.iter().enumerate() {
            let path = dir_entry.path();

            let is_supposed_to_be_scanned = Self::should_file_be_scanned(dir_entry.file_name().to_os_string().to_string_lossy().as_ref());
            if !is_supposed_to_be_scanned {
                debug!("[scan] [{i}/{len}] skipping {path:?}");
                skipped += 1;
                continue;
            }

            debug!("[scan] [{i}/{len}] scanning {path:?}");

            let sha256_hash = cache.fetch_or_compute_file_sha256_hash(path);
            let uuid = if let Some(existing_entry) = library_db.find_entry_by_sha256_hash(&sha256_hash) {
                let existing_uuid = existing_entry.uuid.0;
                debug!("[scan] found duplicate file: sha256: {sha256_hash}, uuid: {existing_uuid}");
                // TODO: record this duplicate file path in the library entry
                existing_uuid
            } else {
                library_db.add(path, sha256_hash, &info.domain)
            };
            index.files.insert(path.to_owned(), uuid.into());
        }

        let scanning_end_timestamp = Instant::now();
        let scanning_duration = scanning_end_timestamp.duration_since(scanning_start_timestamp);

        info!(
            "scanning done; took {scanning_duration:?}; {len} files found: {} files scanned in, {skipped} files skipped",
            len - skipped
        );

        index
    }

    pub fn save(&self, path: &Path) -> Result<(), Error> {
        path.write_as_json_pretty(self)?;
        Ok(())
    }
}
