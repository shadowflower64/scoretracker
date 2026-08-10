use crate::error::CmdError;
use constcat::concat;
use regex::Regex;
use relative_path::RelativePath;
use scoretracker::config::Config;
use scoretracker::data::library::database::MediaCategory;
use scoretracker::data::library::{database::LibraryDatabase, index::LibraryIndex};
use scoretracker::util::filelocked::FileLockableData;
use std::path::PathBuf;
use std::sync::LazyLock;

pub fn identify_media_based_on_relpath(relpath: &RelativePath) -> MediaCategory {
    let filename = relpath.file_name().expect("todo: filename should be the last part of relpath");

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

pub fn automark_library_files(library_dir: PathBuf) -> Result<(), CmdError> {
    let config = Config::load().expect("todo");
    let library_index =
        LibraryIndex::read_without_locking(library_dir.join(LibraryIndex::STANDARD_FILENAME)).map_err(CmdError::LibraryIndexReadError)?;
    let mut library_db =
        LibraryDatabase::lock_and_read(config.library_database_path(), None).map_err(CmdError::LibraryDatabaseOpenError)?;

    for (relpath, uuid) in library_index.files {
        let entry = library_db
            .find_entry_by_uuid_mut(uuid)
            .ok_or(CmdError::LibraryRescanNeeded(uuid.0))?;

        if entry.media_category == MediaCategory::Unspecified {
            entry.media_category = identify_media_based_on_relpath(&relpath);
        }
    }

    library_db.save_and_close().map_err(CmdError::LibraryDatabaseWriteError)?;
    Ok(())
}
