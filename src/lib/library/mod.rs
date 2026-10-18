//! A collection of proof files.
//!
//! A "library" is a directory on a hard drive that contains videos and images
//! that are proof of a player's [performance](crate::data::scoreboard::performance) on a song.
//!
//! Apart from just video and image files, a library directory contains additional files:
//! * `library_info.toml` ([`mod@info`]) - contains basic information about the library, such as the domain name.
//! * `library_cache.json` ([`cache`]) - stores file hashes in relation to file names and file stat, so that the hash doesn't have to be recalculated all the time.
//! * `library_index.json` ([`index`]) - contains a mapping from file paths to proof UUIDs, for easily locating proof files.
//! * `library_aux.json` ([`aux_data`]) - contains additional auxiliary data about the library, such as local tags.
//!
//! There is also a file that is shared across all libraries:
//! * `library_database.json` ([`database`]) - permanently stores all of the data about all of the proofs globally.
//!   This includes: URLs to the proof, comments, manually assigned categories, related performance UUIDs, etc.

pub mod aux_data;
pub mod cache;
pub mod index;
pub mod info;
pub mod root;

use crate::config::toml::{TomlConfig, TomlConfigError};
use crate::data::library::proof::Proof;
use crate::data::library::sha256::Sha256Hash;
use crate::data::library::stpl_url::{LibraryDomain, StplUrl};
use crate::db::Database;
use crate::hive::worker::data::WorkerInfo;
use crate::library::cache::LibraryCache;
use crate::library::index::LibraryIndex;
use crate::library::info::LibraryInfo;
use crate::util::filelocked::{FileLockableDataDefault, FileLocked};
use crate::util::{file_ex, lockfile};
use crate::util::{filelocked::FileLockableData, uuid::UuidString};
use crate::{debug, info, log_fn_name, log_should_print_debug, warn};
use function_name::named;
use relative_path::{PathExt, RelativePath, RelativePathBuf};
use std::collections::{HashMap, HashSet};
use std::path::{self, Path, PathBuf};
use std::time::Instant;
use thiserror::Error;
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

/// Returns the library directory ("repository directory") of the specified file.
///
/// This function searches for the root directory of the library by repeatedly going up a level until it finds a file named `library_info.toml`.
/// The function returns the path of the directory that the "library_info.toml" file is located in.
///
/// Returns [`None`] if the file was not found in any parent directory.
pub fn library_dir_of_path(path: &Path) -> Option<PathBuf> {
    let mut current_path = path.to_path_buf();
    while let Some(parent_path) = current_path.parent() {
        if current_path.join(LibraryInfo::STANDARD_FILENAME).is_file() {
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
/// # use scoretracker::library::path_within_library_dir;
/// # use relative_path::RelativePathBuf;
/// assert_eq!(path_within_library_dir("/mnt/example/videos/library", "/mnt/example/videos/library/example_file_1.mp4"), Some(RelativePathBuf::from("example_file_1.mp4")));
/// assert_eq!(path_within_library_dir("/mnt/example/videos/library", "/mnt/example/videos/library/directory/example_file_2.mp4"), Some(RelativePathBuf::from("directory/example_file_2.mp4")));
/// assert_eq!(path_within_library_dir("../library", "../library/example_file_3.mp4"), Some(RelativePathBuf::from("example_file_3.mp4")));
/// assert_eq!(path_within_library_dir("/mnt/example/videos/library", "/mnt/example/videos/library/directory/inner/../example_file_4.mp4"), Some(RelativePathBuf::from("directory/example_file_4.mp4")));
/// assert_eq!(path_within_library_dir("/mnt/example/videos/library", "/mnt/example/videos/library/directory/../../../../example_file_5.mp4"), None);
/// //assert_eq!(path_within_library_dir(r"C:\Videos\Proof Library", r"C:\Videos\Proof Library\Test Game 1\..\Test Game 2\example_file_6.mp4"), Some(RelativePathBuf::from(r"Test Game 2\example_file_6.mp4")));
/// ```
pub fn path_within_library_dir(library_dir: impl AsRef<Path>, target_file_path: impl AsRef<Path>) -> Option<RelativePathBuf> {
    let library_dir_path = path::absolute(library_dir.as_ref()).ok()?;
    let file_path = path::absolute(target_file_path.as_ref()).ok()?;
    let relative_file_path = file_path.relative_to(library_dir_path).ok()?.normalize();

    if relative_file_path.starts_with("..") {
        None
    } else {
        Some(relative_file_path)
    }
}

pub fn create_stpl_url_to_file(
    library_info: LibraryInfo,
    library_dir: impl AsRef<Path>,
    target_file_path: impl AsRef<Path>,
) -> Option<StplUrl> {
    let rel = path_within_library_dir(library_dir.as_ref(), target_file_path)?;
    Some(create_stpl_url_to_relfile(library_info, rel))
}

pub fn create_stpl_url_to_relfile(library_info: LibraryInfo, target_file_relpath: impl AsRef<RelativePath>) -> StplUrl {
    StplUrl::new(library_info.domain, Some(target_file_relpath.as_ref().to_string()))
}

#[derive(Debug, Error)]
pub enum LibraryScanError {
    #[error("cannot read library info: {0}")]
    CannotReadInfo(TomlConfigError),
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

struct FilteredLibraryFiles {
    /// Filtered list of paths of important files in the library (.mp4, .png, ...)
    file_paths: Vec<RelativePathBuf>,
    all_file_count: usize,
    filtered_files_count: usize,
    skipped_files_count: usize,
}

/// Get all files in directory recursively
#[named]
fn traverse_library_dir(library_dir: &Path) -> Vec<PathBuf> {
    log_fn_name!("library" : auto);
    log_should_print_debug!(VERBOSE_SCANNING);

    info!("looking for files in directory");
    WalkDir::new(library_dir)
        .into_iter()
        .filter_map(|result| result.ok())
        .filter(|dir_entry| dir_entry.file_type().is_file())
        .map(|a| a.path().to_path_buf())
        .collect()
}

#[named]
fn find_specific_in_library_dir(file_paths: &[&Path]) -> Vec<PathBuf> {
    log_fn_name!("library" : auto);
    log_should_print_debug!(VERBOSE_SCANNING);

    info!("looking for files in directory");
    file_paths
        .iter()
        .filter(|path| path.is_file())
        .map(|path| path.to_path_buf())
        .collect()
}

/// Filter the found files, only include ones that are actual video/image files
#[named]
fn filter_down_files(library_dir: &Path, all_files: Vec<PathBuf>) -> FilteredLibraryFiles {
    log_fn_name!("library" : auto);
    log_should_print_debug!(VERBOSE_SCANNING);

    let all_file_count = all_files.len();
    info!("found {all_file_count} files; filtering results");
    let mut skipped_files_count = 0;
    let filtered_file_paths: Vec<RelativePathBuf> = all_files
        .into_iter()
        .filter_map(|denormalized_path| {
            let Some(relative_path) = path_within_library_dir(library_dir, &denormalized_path) else {
                warn!("not within library bounds: {denormalized_path:?} (path_within_library_dir returned None)");
                skipped_files_count += 1;
                return None;
            };

            let is_supposed_to_be_scanned =
                should_file_be_scanned(denormalized_path.file_name().unwrap_or_default().to_string_lossy().as_ref());
            if !is_supposed_to_be_scanned {
                debug!("skipping {relative_path:?}");
                skipped_files_count += 1;
                return None;
            }

            Some(relative_path)
        })
        .collect();

    let filtered_files_count = filtered_file_paths.len();
    info!("filtered down to {filtered_files_count}; fetching cache for sha256 hashes");

    FilteredLibraryFiles {
        file_paths: filtered_file_paths,
        all_file_count,
        filtered_files_count,
        skipped_files_count,
    }
}

struct FetchedHashesFromCache {
    sha256_hashes: HashMap<RelativePathBuf, Option<Sha256Hash>>,
    hashes_to_calculate: usize,
}

/// Scan for any cached sha256 hashes
#[named]
fn fetch_sha256_hashes_from_cache(
    library_dir: &Path,
    file_paths: Vec<RelativePathBuf>,
    filtered_files_count: usize,
    cache: &LibraryCache,
) -> FetchedHashesFromCache {
    log_fn_name!("library" : auto);
    log_should_print_debug!(VERBOSE_SCANNING);
    let mut sha256_hashes_of_all_files = HashMap::new();
    let mut hashes_to_calculate = 0;

    for (i, relative_path) in file_paths.into_iter().enumerate() {
        debug!("[{}/{}] fetching cache for: {relative_path:?}", i + 1, filtered_files_count);
        let cached_hash_opt = cache.fetch_file_sha256_hash(&relative_path.to_path(library_dir));
        if cached_hash_opt.is_none() {
            hashes_to_calculate += 1;
        }
        sha256_hashes_of_all_files.insert(relative_path, cached_hash_opt);
    }
    FetchedHashesFromCache {
        sha256_hashes: sha256_hashes_of_all_files,
        hashes_to_calculate,
    }
}
struct CalculatedHashes {
    sha256_hashes: HashMap<RelativePathBuf, Sha256Hash>,
    calculated_hashes: usize,
    fetched_hashes: usize,
}

/// Calculate remaining file hashes (this takes a long time)
#[named]
fn calculate_missing_sha256_hashes(
    library_dir: &Path,
    cache: &mut FileLocked<LibraryCache>,
    known_sha256_hashes: HashMap<RelativePathBuf, Option<Sha256Hash>>,
    hashes_to_calculate: usize,
) -> Result<CalculatedHashes, LibraryScanError> {
    type E = LibraryScanError;
    log_fn_name!("library" : auto);
    info!("calculating sha256 hashes for {hashes_to_calculate} files (this may take a long time)");
    let mut calculated_hashes = 0;
    let mut cached_hashes = 0;
    let sha256_hashes: HashMap<_, _> = known_sha256_hashes
        .into_iter()
        .map(|(relative_path, cached_hash_opt)| {
            if let Some(cached_sha256_hash) = cached_hash_opt {
                cached_hashes += 1;
                (relative_path, cached_sha256_hash)
            } else {
                info!(
                    "[{}/{}] calculating sha256 hash for: {relative_path:?}",
                    calculated_hashes + 1,
                    hashes_to_calculate
                );
                let calculated_sha256_hash = cache.fetch_or_compute_file_sha256_hash(&relative_path.to_path(library_dir));
                calculated_hashes += 1;
                (relative_path, calculated_sha256_hash)
            }
        })
        .collect();

    // Write all results to cache (if they haven't been already saved by the autosave)
    cache.save_to_file().map_err(E::CannotWriteCache)?;

    Ok(CalculatedHashes {
        sha256_hashes,
        calculated_hashes,
        fetched_hashes: cached_hashes,
    })
}

struct ServerScanResponse {
    proof_uuids: HashMap<RelativePathBuf, UuidString>,
    proofs: Vec<Proof>,
}

impl ServerScanResponse {
    pub fn into_index(self) -> LibraryIndex {
        LibraryIndex { files: self.proof_uuids }
    }
    pub fn add_to_index(&self, index: &mut LibraryIndex) {
        todo!("merge existing index with new info from server: {index:?}");
    }
    pub async fn apply_mutations(&mut self, proof_mutator: impl Fn(&mut Proof)) {
        for proof in &mut self.proofs {
            proof_mutator(proof);
        }

        for proof in &self.proofs {
            // submit changed proofs
            todo!("send PUT request to /api/library/proof")
        }
    }
}
async fn submit_scan_full_to_server(library_info: &LibraryInfo, sha256_hashes: HashMap<RelativePathBuf, Sha256Hash>) -> ServerScanResponse {
    #[derive(Debug)]
    struct RequestBody {
        library_domain: LibraryDomain,
        authenticity_string: String,
        sha256_hashes: HashMap<RelativePathBuf, Sha256Hash>,
    }

    let body = RequestBody {
        library_domain: library_info.domain.clone(),
        authenticity_string: library_info.authenticity_string.clone(),
        sha256_hashes,
    };

    // Send all calculated/cached SHA256 hashes, along with internal filepaths, for files that are currently present in the directory to the server/database. `/api/library/scan_full`
    // Also send credentials, so that we know this is the real library that it claims to be.
    todo!("send POST request to /api/library/scan_full {body:#?}");

    // The server should:
    //     * remove ALL previous references to this library domain in the library URLs from the database,
    //     * add URLs for the specified files (meaning, all old files that used be in this library and are not present anymore will have their URLs removed, and any files currently present will have their URLs updated),
    //     * insert new proof records in the case that a SHA256 hash was not found in the database,
    //     * return proof UUIDs for every file/hash pair in the request.
}

async fn submit_scan_add_to_server(library_info: &LibraryInfo, sha256_hashes: HashMap<RelativePathBuf, Sha256Hash>) -> ServerScanResponse {
    #[derive(Debug)]
    struct RequestBody {
        library_domain: LibraryDomain,
        authenticity_string: String,
        sha256_hashes: HashMap<RelativePathBuf, Sha256Hash>,
    }

    let body = RequestBody {
        library_domain: library_info.domain.clone(),
        authenticity_string: library_info.authenticity_string.clone(),
        sha256_hashes,
    };

    // Send all calculated/cached SHA256 hashes, along with internal filepaths, for files that are currently present in the directory to the server/database. `/api/library/scan_full`
    // Also send credentials, so that we know this is the real library that it claims to be.
    todo!("send POST request to /api/library/scan_add {body:#?}");

    // The server should:
    //     * remove ALL previous references to this library domain in the library URLs from the database,
    //     * add URLs for the specified files (meaning, all old files that used be in this library and are not present anymore will have their URLs removed, and any files currently present will have their URLs updated),
    //     * insert new proof records in the case that a SHA256 hash was not found in the database,
    //     * return proof UUIDs for every file/hash pair in the request.
}

/// Fully scan a library directory for added/moved/removed files.
///
/// Scanning a library is a multi-step process, and it involves updating the [`LibraryIndex`], the [`LibraryCache`], and the [`LibraryDatabase`].
/// To scan a folder, this function does the following steps:
/// 1. Read the [`LibraryInfo`] file within the library directory.
/// 2. Read the [`LibraryCache`] file into memory - it will be used to skip recalculating SHA256 hashes for known files.
/// 3. Walk through all files in the library directory, finding all candidate files. (files with a video/image extension)
/// 4. Fetch SHA256 hashes for known files from the candidate list. Calculate the amount of files for which the hash is not known for progress-reporting reasons.
/// 5. Compute SHA256 hashes for all files that are not known. This takes a long time! (Autosave results after every file, so the process can be resumed at any time.)
/// 6. Now that we have all SHA256 hashes for files that are currently in the library directory, send all internal filepaths and hashes to the server.
/// - The server has to remove any existing proof URLs that have the domain of this database from all database entries, and add fresh ones; and return proof UUIDs (new or existing) for all requested files to the client.
/// 7. Generate a completely new [`LibraryIndex`] file with UUIDs fetched from the database, returned by the server.
#[named]
pub async fn library_scan_full(library_dir: &Path) -> Result<(), LibraryScanError> {
    type E = LibraryScanError;
    log_fn_name!("library" : auto);

    let scanning_start_timestamp = Instant::now();

    let library_info = LibraryInfo::load_from_file(library_dir.join(LibraryInfo::STANDARD_FILENAME)).map_err(E::CannotReadInfo)?;
    let mut cache = LibraryCache::lock_and_read_or_default(
        library_dir.join(LibraryCache::STANDARD_FILENAME),
        None, /* TODO: is this correct? */
    )
    .map_err(E::CannotReadCache)?;

    let all_files = traverse_library_dir(library_dir);
    let filtered = filter_down_files(library_dir, all_files);
    let fetched = fetch_sha256_hashes_from_cache(library_dir, filtered.file_paths, filtered.filtered_files_count, &cache);
    let hashes = calculate_missing_sha256_hashes(library_dir, &mut cache, fetched.sha256_hashes, fetched.hashes_to_calculate)?;

    let res = submit_scan_full_to_server(&library_info, hashes.sha256_hashes).await;
    let index = res.into_index();
    index
        .save(&library_dir.join(LibraryIndex::STANDARD_FILENAME))
        .map_err(E::CannotWriteReplaceIndex)?;

    let scanning_end_timestamp = Instant::now();
    let scanning_duration = scanning_end_timestamp.duration_since(scanning_start_timestamp);

    info!(
        "scanning done; took {scanning_duration:?}; {} files found in directory: {} files scanned in, {} files skipped, {} new hashes calculated, {} existing hashes fetched",
        filtered.all_file_count,
        filtered.filtered_files_count,
        filtered.skipped_files_count,
        hashes.calculated_hashes,
        hashes.fetched_hashes,
    );
    Ok(())
}

/// Scan-in specific files within the library dir, without traversing through the entire library directory.
#[named]
pub async fn library_scan_add_files(
    library_dir: &Path,
    file_paths: &[&Path],
    proof_mutator: impl Fn(&mut Proof),
) -> Result<HashMap<RelativePathBuf, UuidString>, LibraryScanError> {
    type E = LibraryScanError;
    log_fn_name!("library" : auto);

    let scanning_start_timestamp = Instant::now();

    let library_info = LibraryInfo::load_from_file(library_dir.join(LibraryInfo::STANDARD_FILENAME)).map_err(E::CannotReadInfo)?;
    let mut cache = LibraryCache::lock_and_read_or_default(
        library_dir.join(LibraryCache::STANDARD_FILENAME),
        None, /* TODO: is this correct? */
    )
    .map_err(E::CannotReadCache)?;

    let all_files = find_specific_in_library_dir(file_paths);
    let filtered = filter_down_files(library_dir, all_files);
    let fetched = fetch_sha256_hashes_from_cache(library_dir, filtered.file_paths, filtered.filtered_files_count, &cache);
    let hashes = calculate_missing_sha256_hashes(library_dir, &mut cache, fetched.sha256_hashes, fetched.hashes_to_calculate)?;

    let mut index = LibraryIndex::lock_and_read_or_default(
        library_dir.join(LibraryIndex::STANDARD_FILENAME),
        None, /* TODO: is this correct? */
    )
    .expect("todo");
    let mut res = submit_scan_add_to_server(&library_info, hashes.sha256_hashes).await;
    res.add_to_index(&mut index.inner);
    index.save_and_unlock().map_err(E::CannotWriteIndex)?;

    res.apply_mutations(proof_mutator).await;

    let scanning_end_timestamp = Instant::now();
    let scanning_duration = scanning_end_timestamp.duration_since(scanning_start_timestamp);

    info!(
        "scanning (add) done; took {scanning_duration:?}; {} files found in directory: {} files scanned in, {} files skipped, {} new hashes calculated, {} existing hashes fetched",
        filtered.all_file_count,
        filtered.filtered_files_count,
        filtered.skipped_files_count,
        hashes.calculated_hashes,
        hashes.fetched_hashes,
    );

    Ok(res.proof_uuids)
}

pub async fn scan_register_added_file(
    library_dir: &Path,
    file_path: &Path,
    proof_mutator: impl Fn(&mut Proof),
) -> Result<(RelativePathBuf, UuidString), LibraryScanError> {
    let pairs = library_scan_add_files(library_dir, &vec![file_path], proof_mutator).await?;
    let pair = pairs
        .iter()
        .next()
        .map(|(path, uuid)| (path.to_owned(), uuid.to_owned()))
        .expect("todo error handling");
    Ok(pair)
}

#[named]
pub async fn scan_register_removed_files(library_dir: &Path, file_paths: Vec<&Path>) -> Result<(), LibraryScanError> {
    todo!("not done");
}

pub async fn scan_register_removed_file(library_dir: &Path, library_db_path: &Path, file_path: &Path) -> Result<(), LibraryScanError> {
    scan_register_removed_files(library_dir, vec![file_path]).await
}

#[named]
pub fn sync_library_index_with_db_essence(
    library_dir: &Path,
    library_index: &LibraryIndex,
    db: &Database,
    library_domain: LibraryDomain,
    worker_info: Option<&WorkerInfo>,
) -> Result<(), LibraryScanError> {
    type E = LibraryScanError;
    log_fn_name!("library" : auto);

    let mut reverse_index: HashMap<UuidString, Vec<RelativePathBuf>> = HashMap::new();
    let mut unused_proof_uuids_in_index = HashSet::new();

    for (relative_path, proof_uuid) in &library_index.files {
        unused_proof_uuids_in_index.insert(proof_uuid);
        if let Some(files_for_this_proof) = reverse_index.get_mut(&proof_uuid) {
            files_for_this_proof.push(relative_path.to_owned());
        } else {
            reverse_index.insert(proof_uuid.to_owned(), vec![relative_path.to_owned()]);
        }
    }

    todo!();
    /*
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
    */
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
    let library_info = LibraryInfo::load_from_file(library_dir.join(LibraryInfo::STANDARD_FILENAME)).map_err(E::CannotReadInfo)?;
    let library_domain = library_info.domain;
    let library_index =
        LibraryIndex::read_without_locking(library_dir.join(LibraryIndex::STANDARD_FILENAME)).map_err(E::CannotReadIndex)?;

    let db = todo!();
    sync_library_index_with_db_essence(library_dir, &library_index, db, library_domain, worker_info)
}
