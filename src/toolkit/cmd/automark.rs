use crate::error::CmdError;
use constcat::concat;
use function_name::named;
use regex::Regex;
use relative_path::RelativePath;
use scoretracker::config::Config;
use scoretracker::data::library::database::MediaCategory;
use scoretracker::data::library::{database::LibraryDatabase, index::LibraryIndex};
use scoretracker::util::filelocked::FileLockableData;
use scoretracker::{info, log_fn_name, success};
use std::path::PathBuf;
use std::sync::LazyLock;

pub fn identify_media_based_on_relpath(relpath: &RelativePath) -> MediaCategory {
    let Some(filename) = relpath.file_name() else {
        return MediaCategory::Unspecified;
    };

    pub const OBS_REPLAY: &str = r"^Replay \d{4}-\d{2}-\d{2} \d{2}-\d{2}-\d{2}";
    pub const OBS_REC: &str = r"^\d{4}-\d{2}-\d{2} \d{2}-\d{2}-\d{2}";
    pub const MOBILE_REC: &str = r"^Record_\d{4}-\d{2}-\d{2}-\d{2}-\d{2}-\d{2}_[0-9a-f]{32}";
    pub const AUTOCUT_SUFFIX: &str = r" cut";
    pub const LLC_SUFFIX: &str = r"-\d{2}\.\d{2}\.\d{2}\.\d{3}-\d{2}\.\d{2}\.\d{2}\.\d{3}(-seg\d+)?";

    static OBS_RECORDING_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(concat!(OBS_REC, r"\.mkv$")).expect("could not compile regex"));
    static OBS_RECORDING_AUTOCUT_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(concat!(OBS_REC, AUTOCUT_SUFFIX, r"\.mkv$")).expect("could not compile regex"));
    static OBS_RECORDING_LLC_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(concat!(OBS_REC, LLC_SUFFIX, r"\.mkv$")).expect("could not compile regex"));

    static OBS_REPLAY_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(concat!(OBS_REPLAY, r"\.mkv$")).expect("could not compile regex"));
    static OBS_REPLAY_AUTOCUT_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(concat!(OBS_REPLAY, AUTOCUT_SUFFIX, r"\.mkv$")).expect("could not compile regex"));
    static OBS_REPLAY_LLC_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(concat!(OBS_REPLAY, LLC_SUFFIX, r"\.mkv$")).expect("could not compile regex"));

    static MOBILE_SCREEN_RECORDING_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(concat!(MOBILE_REC, r"\.mp4$")).expect("could not compile regex"));
    static MOBILE_SCREEN_RECORDING_AUTOCUT_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(concat!(MOBILE_REC, AUTOCUT_SUFFIX, r"\.mp4$")).expect("could not compile regex"));
    static MOBILE_SCREEN_RECORDING_LLC_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(concat!(MOBILE_REC, LLC_SUFFIX, r"\.mp4$")).expect("could not compile regex"));

    if OBS_RECORDING_REGEX.is_match(filename) {
        MediaCategory::ObsRecording
    } else if OBS_RECORDING_AUTOCUT_REGEX.is_match(filename) {
        unimplemented!("OBS_RECORDING_AUTOCUT_REGEX matched but MediaCategory::ObsRecordingAutocut doesn't exist")
    } else if OBS_RECORDING_LLC_REGEX.is_match(filename) {
        MediaCategory::ObsRecordingLosslessCut
    } else if OBS_REPLAY_REGEX.is_match(filename) {
        MediaCategory::ObsReplay
    } else if OBS_REPLAY_AUTOCUT_REGEX.is_match(filename) {
        MediaCategory::ObsReplayAutocut
    } else if OBS_REPLAY_LLC_REGEX.is_match(filename) {
        MediaCategory::ObsReplayLosslessCut
    } else if MOBILE_SCREEN_RECORDING_REGEX.is_match(filename) {
        MediaCategory::MobileScreenRecording
    } else if MOBILE_SCREEN_RECORDING_AUTOCUT_REGEX.is_match(filename) {
        unimplemented!("MOBILE_SCREEN_RECORDING_AUTOCUT_REGEX matched but MediaCategory::MobileScreenRecordingAutocut doesn't exist")
    } else if MOBILE_SCREEN_RECORDING_LLC_REGEX.is_match(filename) {
        MediaCategory::MobileScreenRecordingLosslessCut
    } else {
        MediaCategory::Unspecified
    }
}

#[named]
pub fn automark_library_files(library_dir: PathBuf) -> Result<(), CmdError> {
    log_fn_name!(auto);

    info!("reading database");
    let config = Config::load().map_err(CmdError::ConfigReadError)?;
    let library_index =
        LibraryIndex::read_without_locking(library_dir.join(LibraryIndex::STANDARD_FILENAME)).map_err(CmdError::LibraryIndexReadError)?;
    let mut library_db =
        LibraryDatabase::lock_and_read(config.library_database_path(), None).map_err(CmdError::LibraryDatabaseOpenError)?;

    info!("automarking {} files...", library_index.files.len());
    let mut counter_modified = 0;
    let mut counter_unmodified = 0;

    for (relpath, uuid) in library_index.files {
        let entry = library_db
            .find_entry_by_uuid_mut(uuid.0)
            .ok_or(CmdError::LibraryRescanNeeded(uuid.0))?;

        if entry.media_category == MediaCategory::Unspecified {
            let new = identify_media_based_on_relpath(&relpath);
            success!("specified as: {new:?}; path: {relpath:?}");
            entry.media_category = new;
            counter_modified += 1;
        } else {
            info!("was already specified: {:?}; path: {relpath:?}", entry.media_category);
            counter_unmodified += 1;
        }
    }

    info!("saving database");
    library_db.save_and_close().map_err(CmdError::LibraryDatabaseWriteError)?;

    success!("automarked {counter_modified} new files, {counter_unmodified} were already specified");
    Ok(())
}
