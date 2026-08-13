//! A collection of proof files.
//!
//! A "library" is a directory on a hard drive that contains videos and images
//! that are proof of a player's [performance](crate::data::scoreboard::performance) on a song.
//!
//! Apart from just video and image files, a library directory contains additional files:
//! * `library_info.json` ([`mod@info`]) - contains basic information about the library, such as the domain name.
//! * `library_cache.json` ([`cache`]) - stores file hashes in relation to file names and file stat, so that the hash doesn't have to be recalculated all the time.
//! * `library_index.json` ([`index`]) - contains a mapping from file paths to proof UUIDs, for easily locating proof files.
//! * `library_aux.json` ([`aux_data`]) - contains additional auxiliary data about the library, such as local tags.
//!
//! There is also a file that is shared across all libraries:
//! * `library_database.json` ([`database`]) - permanently stores all of the data about all of the proofs globally.
//!   This includes: URLs to the proof, comments, manually assigned categories, related performance UUIDs, etc.

pub mod aux_data;
pub mod cache;
pub mod database;
pub mod index;
pub mod info;
pub mod stpl_url;

use crate::data::library::cache::LibraryCache;
use crate::data::library::database::{LibraryDatabase, LibraryEntry};
use crate::data::library::index::LibraryIndex;
use crate::data::library::info::LibraryInfo;
use crate::data::library::stpl_url::{LibraryDomain, StplUrl};
use crate::hive::worker::data::WorkerInfo;
use crate::util::filelocked::{FileLockableDataDefault, FileLocked};
use crate::util::{file_ex, lockfile};
use crate::util::{filelocked::FileLockableData, uuid::UuidString};
use crate::{debug, info, log_fn_name, log_should_print_debug, warn};
use relative_path::{PathExt, RelativePath, RelativePathBuf};
use std::collections::{HashMap, HashSet};
use std::path::{self, Path, PathBuf};
use std::time::Instant;
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

/// Returns the library directory ("repository directory") of the specified file.
///
/// This function searches for the root directory of the library by repeatedly going up a level until it finds a file named `library_info.json`.
/// The function returns the path of the directory that the "library_info.json" file is located in.
///
/// Returns [`None`] if the file was not found in any parent directory.
pub fn get_library_dir_of_path(path: &Path) -> Option<PathBuf> {
    let mut current_path = path.to_path_buf();
    while let Some(parent_path) = current_path.parent() {
        if current_path.join("library_info.json").is_file() {
            return Some(current_path);
        }
        current_path = parent_path.to_path_buf();
    }
    None
}

/// Creates a normalized path to a file within the library.
///
/// Creates a [`RelativePathBuf`] that represents a path within the library dir to the specified file.
///
/// Returns [`None`] if the file is not within the library or the process failed in some other way.
///
/// # Examples
/// ```
/// # use scoretracker::data::library::path_within_library_dir;
/// # use relative_path::RelativePathBuf;
/// assert_eq!(path_within_library_dir("/mnt/example/videos/library", "/mnt/example/videos/library/example_file_1.mp4"), Some(RelativePathBuf::from("example_file_1.mp4")));
/// assert_eq!(path_within_library_dir("/mnt/example/videos/library", "/mnt/example/videos/library/directory/example_file_2.mp4"), Some(RelativePathBuf::from("directory/example_file_2.mp4")));
/// assert_eq!(path_within_library_dir("../library", "../library/example_file_3.mp4"), Some(RelativePathBuf::from("example_file_3.mp4")));
/// assert_eq!(path_within_library_dir("/mnt/example/videos/library", "/mnt/example/videos/library/directory/inner/../example_file_4.mp4"), Some(RelativePathBuf::from("directory/example_file_4.mp4")));
/// assert_eq!(path_within_library_dir("/mnt/example/videos/library", "/mnt/example/videos/library/directory/../../../../example_file_5.mp4"), None);
/// //assert_eq!(path_within_library_dir(r"C:\Videos\Proof Library", r"C:\Videos\Proof Library\Test Game 1\..\Test Game 2\example_file_6.mp4"), Some(RelativePathBuf::from(r"Test Game 2\example_file_6.mp4")));
/// ```
pub fn path_within_library_dir<P1: AsRef<Path>, P2: AsRef<Path>>(library_dir: P1, target_file_path: P2) -> Option<RelativePathBuf> {
    let library_dir_path = path::absolute(library_dir.as_ref()).ok()?;
    let file_path = path::absolute(target_file_path.as_ref()).ok()?;
    let relative_file_path = file_path.relative_to(library_dir_path).ok()?.normalize();

    if relative_file_path.starts_with("..") {
        None
    } else {
        Some(relative_file_path)
    }
}

pub fn create_stpl_url_to_file<P1: AsRef<Path>, P2: AsRef<Path>>(
    library_info: LibraryInfo,
    library_dir: P1,
    target_file_path: P2,
) -> Option<StplUrl> {
    let rel = path_within_library_dir(library_dir.as_ref(), target_file_path)?;
    Some(create_stpl_url_to_relfile(library_info, rel))
}

pub fn create_stpl_url_to_relfile<P: AsRef<RelativePath>>(library_info: LibraryInfo, target_file_relpath: P) -> StplUrl {
    StplUrl::new(
        LibraryDomain::Local(library_info.domain),
        Some(target_file_relpath.as_ref().to_string()),
    )
}

#[derive(Debug, Error)]
pub enum LibraryScanError {
    #[error("cannot read library info: {0}")]
    CannotReadInfo(file_ex::Error),
    #[error("cannot read library index: {0}")]
    CannotReadIndex(file_ex::Error),
    #[error("cannot open library index: {0}")]
    CannotOpenIndex(lockfile::Error),
    #[error("cannot write library index: {0}")]
    CannotWriteReplaceIndex(file_ex::Error),
    #[error("cannot write library index: {0}")]
    CannotWriteIndex(lockfile::Error),
    #[error("cannot read library cache: {0}")]
    CannotReadCache(lockfile::Error),
    #[error("cannot write library cache: {0}")]
    CannotWriteCache(lockfile::Error),
    #[error("cannot open library database: {0}")]
    CannotOpenDatabase(lockfile::Error),
    #[error("cannot write library database: {0}")]
    CannotWriteDatabase(lockfile::Error),
}

/// Determines whether a file should be added to the library scan list or not.
///
/// Returns true if the file should be scanned into the library as proof.
pub fn should_file_be_scanned(filename: &str) -> bool {
    filename.ends_with(".mp4") || filename.ends_with(".mkv")
}

pub const VERBOSE_SCANNING: bool = false;

/// Fully scan a library directory for added/moved/removed files.
///
/// Scanning a library is a multi-step process, and it involves updating the [`LibraryIndex`], the [`LibraryCache`], and the [`LibraryDatabase`].
/// To scan a folder, this function does the following steps:
/// 1. Read the [`LibraryInfo`] file within the library directory. This file will not be written to and so can be read without locking.
/// 2. Read the [`LibraryCache`] file into memory - it will be used to skip recalculating SHA256 hashes for known files.
/// 3. Walk through all files in the library directory, finding all candidate files. (files with a video/image extension)
/// 4. Fetch SHA256 hashes for known files from the candidate list. Calculate the amount of files for which the hash is not known for progress-reporting reasons.
/// 5. Compute SHA256 hashes for all files that are not known. This takes a long time!
/// 6. Now that we have all SHA256 hashes for files in the directory, search for them in the [`LibraryDatabase`] and fetch the proof UUID, or generate a new one if the hash does not exist in the database.
/// 7. Generate a completely new [`LibraryIndex`] file with path paths and UUIDs fetched from the database.
/// 8. Synchronize with the database; iterate through *every* database entry, remove any existing proof URLs that have the domain of this database, and add fresh ones.
pub fn scan_full(library_dir: &Path, library_db_path: &Path, worker_info: Option<&WorkerInfo>) -> Result<(), LibraryScanError> {
    // let info = LibraryInfo::read_without_locking(library_dir.join(LibraryInfo::STANDARD_FILENAME)).expect("could not read library info");
    type E = LibraryScanError;

    log_fn_name!("library:scan_full");
    log_should_print_debug!(VERBOSE_SCANNING);

    let scanning_start_timestamp = Instant::now();

    let library_info = LibraryInfo::read_without_locking(library_dir.join(LibraryInfo::STANDARD_FILENAME)).map_err(E::CannotReadInfo)?;
    let library_domain = LibraryDomain::Local(library_info.domain);

    let mut index = LibraryIndex::default();
    let mut cache =
        LibraryCache::lock_and_read_or_default(library_dir.join(LibraryCache::STANDARD_FILENAME), None).map_err(E::CannotReadCache)?;

    let mut skipped = 0;

    // Get all files in directory recursively
    info!("[scan] looking for files in directory");
    let files_in_dir: Vec<DirEntry> = WalkDir::new(library_dir)
        .into_iter()
        .filter_map(|result| result.ok().filter(|dir_entry| dir_entry.file_type().is_file()))
        .collect();
    let total_files_in_dir = files_in_dir.len();

    // Filter the found files, only include ones that are actual video/image files
    info!("[scan] found {total_files_in_dir} files; filtering results");
    let files_to_scan: Vec<RelativePathBuf> = files_in_dir
        .iter()
        .filter_map(|dir_entry| {
            let denormalized_path = dir_entry.path();
            let Some(relative_path) = path_within_library_dir(library_dir, denormalized_path) else {
                warn!("[scan] not within library bounds: {denormalized_path:?} (path_within_library_dir returned None)");
                skipped += 1;
                return None;
            };

            let is_supposed_to_be_scanned = should_file_be_scanned(dir_entry.file_name().to_os_string().to_string_lossy().as_ref());
            if !is_supposed_to_be_scanned {
                debug!("[scan] skipping {relative_path:?}");
                skipped += 1;
                return None;
            }

            Some(relative_path)
        })
        .collect();
    let total_files_to_scan = files_to_scan.len();

    let mut sha256_hashes = HashMap::new();
    let mut hashes_to_calculate = 0;

    // Scan for any cached sha256 hashes
    info!("[scan] filtered down to {total_files_to_scan}; fetching cache for sha256 hashes");
    for (i, relative_path) in files_to_scan.iter().enumerate() {
        debug!("[scan] [{}/{}] fetching cache for: {relative_path:?}", i + 1, total_files_to_scan);
        let cached_hash_opt = cache.fetch_file_sha256_hash(&relative_path.to_path(library_dir));
        if cached_hash_opt.is_none() {
            hashes_to_calculate += 1;
        }
        sha256_hashes.insert(relative_path, cached_hash_opt);
    }

    // Calculate remaining ones (this takes a long time, so the db is closed during this step)
    info!("[scan] calculating sha256 hashes for {hashes_to_calculate} files (this may take a long time)");
    let mut calculated_hashes = 0;
    let mut cached_hashes = 0;
    let sha256_hashes: HashMap<_, _> = sha256_hashes
        .iter()
        .map(|(relative_path, cached_hash_opt)| {
            if let Some(cached_sha256_hash) = cached_hash_opt {
                cached_hashes += 1;
                (*relative_path, cached_sha256_hash.to_owned())
            } else {
                info!(
                    "[scan] [{}/{}] calculating sha256 hash for: {relative_path:?}",
                    calculated_hashes + 1,
                    hashes_to_calculate
                );
                let calculated_sha256_hash = cache.fetch_or_compute_file_sha256_hash(&relative_path.to_path(library_dir));
                calculated_hashes += 1;
                (*relative_path, calculated_sha256_hash)
            }
        })
        .collect();

    // Write all results to cache (if they haven't been already saved by the autosave)
    cache.save_and_unlock().map_err(E::CannotWriteCache)?;

    // Look up proof UUIDs in the database, and insert new proof entries in the database if no entry with the given sha256 hash is found.
    info!("[scan] getting uuids from database");
    let mut library_db = LibraryDatabase::lock_and_read(library_db_path, worker_info).map_err(E::CannotOpenDatabase)?;
    let len = sha256_hashes.len();
    for (i, (relative_path, sha256_hash)) in sha256_hashes.iter().enumerate() {
        let (uuid, existed) = library_db.fetch_or_insert(relative_path, sha256_hash.clone(), library_domain.clone());
        if existed {
            debug!(
                "[scan] [{}/{}] fetching uuid for existing database entry ({uuid}) for: {relative_path:?}",
                i + 1,
                len
            );
        } else {
            debug!(
                "[scan] [{}/{}] adding new database entry ({uuid}) for: {relative_path:?}",
                i + 1,
                len
            );
        }

        index.files.insert((*relative_path).clone(), uuid.into());
    }
    index
        .save(&library_dir.join(LibraryIndex::STANDARD_FILENAME))
        .map_err(E::CannotWriteReplaceIndex)?;

    // Now sync all of the URLs within the index file with the database
    info!("[scan] syncing database");
    sync_library_index_with_db_essence(library_dir, index, |_| Ok(library_db), library_domain, worker_info)?;

    let scanning_end_timestamp = Instant::now();
    let scanning_duration = scanning_end_timestamp.duration_since(scanning_start_timestamp);

    info!(
        "[scan] scanning done; took {scanning_duration:?}; {total_files_in_dir} files found in directory: {total_files_to_scan} files scanned in, {skipped} files skipped, {calculated_hashes} new hashes calculated, {cached_hashes} existing hashes fetched"
    );

    Ok(())
}

pub fn scan_register_added_files(
    _library_dir: &Path,
    _library_db_path: &Path,
    _file_paths: Vec<&Path>,
    _entry_mutator: impl Fn(&mut LibraryEntry),
    _worker_info: Option<&WorkerInfo>,
) -> Result<Vec<UuidString>, LibraryScanError> {
    //log_fn_name!("library:scan_register_added_files");

    todo!()
}

pub fn scan_register_added_file(
    library_dir: &Path,
    library_db_path: &Path,
    file_path: &Path,
    entry_mutator: impl Fn(&mut LibraryEntry),
    worker_info: Option<&WorkerInfo>,
) -> Result<UuidString, LibraryScanError> {
    Ok(
        *scan_register_added_files(library_dir, library_db_path, vec![file_path], entry_mutator, worker_info)?
            .first()
            .unwrap(),
    )
}

pub fn scan_register_removed_files(
    library_dir: &Path,
    library_db_path: &Path,
    file_paths: Vec<&Path>,
    worker_info: Option<&WorkerInfo>,
) -> Result<(), LibraryScanError> {
    type E = LibraryScanError;
    log_fn_name!("library:scan_register_removed_files");

    let library_info = LibraryInfo::read_without_locking(library_dir.join(LibraryInfo::STANDARD_FILENAME)).map_err(E::CannotReadInfo)?;
    let library_domain = LibraryDomain::Local(library_info.domain);

    let mut library_index =
        LibraryIndex::lock_and_read(library_dir.join(LibraryIndex::STANDARD_FILENAME), worker_info).map_err(E::CannotOpenIndex)?;
    let mut library_db = LibraryDatabase::lock_and_read(library_db_path, worker_info).map_err(E::CannotOpenDatabase)?;

    for file_path in file_paths {
        let Some(relpath) = path_within_library_dir(library_dir, file_path) else {
            warn!("file path is not within library: library dir: {library_dir:?}, file: {file_path:?}; skipping...");
            continue;
        };

        if let Some(proof_uuid) = library_index.files.remove(&relpath) {
            if let Some(entry) = library_db.find_entry_by_uuid_mut(proof_uuid) {
                let removed_file_url = StplUrl::new(library_domain.clone(), Some(relpath.to_string()));
                entry.library_urls.retain(|x| *x != removed_file_url);
            } else {
                warn!("proof with uuid was not found in database: {proof_uuid}");
            }
        } else {
            warn!("tried to remove a file from index, but the file was not in the index in the first place: {relpath:?}");
        };
    }

    library_index.save_and_unlock().map_err(E::CannotWriteIndex)?;
    library_db.save_and_unlock().map_err(E::CannotWriteDatabase)?;
    Ok(())
}

pub fn scan_register_removed_file(
    library_dir: &Path,
    library_db_path: &Path,
    file_path: &Path,
    worker_info: Option<&WorkerInfo>,
) -> Result<(), LibraryScanError> {
    scan_register_removed_files(library_dir, library_db_path, vec![file_path], worker_info)
}

pub fn sync_library_index_with_db_essence(
    library_dir: &Path,
    library_index: LibraryIndex,
    library_db_conn: impl FnOnce(Option<&WorkerInfo>) -> lockfile::Result<FileLocked<LibraryDatabase>>,
    library_domain: LibraryDomain,
    worker_info: Option<&WorkerInfo>,
) -> Result<(), LibraryScanError> {
    type E = LibraryScanError;
    log_fn_name!("library:sync_library_index_with_db");

    let mut reverse_index: HashMap<UuidString, Vec<RelativePathBuf>> = HashMap::new();
    let mut unused_proof_uuids_in_index = HashSet::new();

    for (relative_path, proof_uuid) in library_index.files {
        unused_proof_uuids_in_index.insert(proof_uuid);
        if let Some(files_for_this_proof) = reverse_index.get_mut(&proof_uuid) {
            files_for_this_proof.push(relative_path);
        } else {
            reverse_index.insert(proof_uuid, vec![relative_path]);
        }
    }

    let mut library_db = library_db_conn(worker_info).map_err(E::CannotOpenDatabase)?;

    for entry in library_db.entries.iter_mut() {
        // Remove all old URLs that reference this library, without touching all of the other ones.
        entry.library_urls.retain(|url| url.domain != library_domain);

        // Add all URLs that are contained within this library.
        if let Some(files_for_this_proof) = reverse_index.get_mut(&entry.uuid) {
            entry.library_urls.extend(
                files_for_this_proof
                    .iter()
                    .map(|relative_path| StplUrl::new(library_domain.clone(), Some(relative_path.to_string()))),
            );
            unused_proof_uuids_in_index.remove(&entry.uuid);
        }
    }
    library_db.save_and_unlock().map_err(E::CannotWriteDatabase)?;

    // Check if any of the proof UUIDs that are in the index were not present in the database
    for absent_uuid in unused_proof_uuids_in_index {
        if let Some(files) = reverse_index.get(&absent_uuid) {
            warn!(
                "database does not contain proof entry with uuid: {absent_uuid}; the following files are associated with this uuid in the index: {:?}",
                files.iter().map(|file_path| file_path.to_path(library_dir))
            );
        } else {
            warn!("database does not contain proof entry with uuid: {absent_uuid}");
        }
    }

    Ok(())
}

/// Synchronize paths stored in the [`LibraryIndex`] with the `stpl://` URLs stored in the [`LibraryDatabase`].
///
/// This function reads the library directory and updates all URLs within the database that contain the domain name of this library.
/// It removes every old URL in the database that references this library, and adds new URLs to everything that is currently in the library index.
///
/// This function also reports any proof UUIDs that are present in the index file, but absent in the database - these are reported as warnings.
pub fn sync_library_index_with_db(
    library_dir: &Path,
    library_db_path: &Path,
    worker_info: Option<&WorkerInfo>,
) -> Result<(), LibraryScanError> {
    type E = LibraryScanError;
    let library_info = LibraryInfo::read_without_locking(library_dir.join(LibraryInfo::STANDARD_FILENAME)).map_err(E::CannotReadInfo)?;
    let library_domain = LibraryDomain::Local(library_info.domain);

    let library_index =
        LibraryIndex::read_without_locking(library_dir.join(LibraryIndex::STANDARD_FILENAME)).map_err(E::CannotReadIndex)?;

    sync_library_index_with_db_essence(
        library_dir,
        library_index,
        |worker_info| LibraryDatabase::lock_and_read(library_db_path, worker_info),
        library_domain,
        worker_info,
    )
}

pub fn remove_library_domain_from_db(
    library_domain: LibraryDomain,
    library_db_path: &Path,
    worker_info: Option<&WorkerInfo>,
) -> Result<(), LibraryScanError> {
    type E = LibraryScanError;
    let mut library_db = LibraryDatabase::lock_and_read(library_db_path, worker_info).map_err(E::CannotOpenDatabase)?;

    for entry in library_db.entries.iter_mut() {
        // Remove all old URLs that reference this library, without touching all of the other ones.
        entry.library_urls.retain(|url| url.domain != library_domain);
    }
    library_db.save_and_unlock().map_err(E::CannotWriteDatabase)?;
    Ok(())
}
