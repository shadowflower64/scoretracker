//! Library database file handling.
//!
//! A library database file is a file shared globally across libraries, that maps "proof UUIDs" to actual information and metadata about the proof.
//! Every entry in a library database file contains information about the SHA256 hash of the proof file, the type of the file (recording, screenshot etc.),
//! the modification timestamps of the file, the state of the file (is it linked to any score? is it uploaded?), as well as other information.
use crate::data::library::stpl_url::{LibraryDomain, StplUrl};
use crate::util::file_ex::{self, FileEx};
use crate::util::filelocked::FileLockableData;
use crate::util::relative_path_from_segments;
use crate::util::timestamp::{NsDuration, NsLocalTimestamp, NsTimestamp};
use crate::util::uuid::UuidString;
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use thiserror::Error;
use uuid::Uuid;

/// Basic metadata about the file from the `stat` command.
///
/// This struct stores basic metadata about the file, such as the file's size, the file modification time, and the file creation time.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FileStat {
    /// Size of the file, in bytes.
    pub size: u64,

    /// Birth of the file - when was this file created on the disk?
    ///
    /// For raw video files, this is usually the time when the video has started recording.
    pub timestamp_birth: NsTimestamp,

    /// Access of the file - when was this file last accessed or read?
    pub timestamp_access: NsTimestamp,

    /// Modification - when was the data inside of this file modified? For raw video files, this is usually the time when the video has finished recording.
    ///
    /// This value may be set by tools such as LosslessCut to indicate a video recording timestamp, however it may be wrong.
    /// I think LosslessCut actually moves the timestamp wrongly.
    pub timestamp_modification: NsTimestamp,

    /// Status change - when were the permissions(?) changed for this file?
    pub timestamp_status_change: NsTimestamp,

    /// Timestamp of when was the file stat was read (This is not actually part of the `stat` command, and it is stored manually.)
    pub last_check: NsTimestamp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MediaCategory {
    /// Default value - value not selected by user yet.
    #[default]
    #[serde(alias = "unset")] // temp alias for migration while testing, can be removed later
    Unspecified,

    /// An image of the screen captured from a PC.
    PCScreenshot,

    /// An image of the screen captured from a phone.
    MobileScreenshot,

    /// An image captured by a photo camera, a phone camera, or a webcam.
    CameraPhoto,

    /// A video of the screen captured by OBS Studio.
    ObsRecording,

    /// A video of the screen captured by OBS Studio, and then cut using the `autocut` script.
    ObsRecordingAutocut,

    /// A video of the screen captured by OBS Studio, and then cut using LosslessCut.
    ObsRecordingLosslessCut,

    /// A video of the screen captured by OBS Studio using the "Replay Buffer" feature.
    ObsReplay,

    /// A video of the screen captured by OBS Studio using the "Replay Buffer" feature, and then cut using the `autocut` script.
    ObsReplayAutocut,

    /// A video of the screen captured by OBS Studio using the "Replay Buffer" feature, and then cut using LosslessCut.
    ObsReplayLosslessCut,

    /// A video of the screen captured by a phone's screen recording software.
    MobileScreenRecording,

    /// A video of the screen captured by a phone's screen recording software, and then cut using LosslessCut.
    MobileScreenRecordingLosslessCut,

    /// A video captured by a photo camera, a phone camera, or a webcam.
    CameraVideo,

    /// Other media, that doesn't belong to any other category.
    Other,
}

pub type GameId = String;
pub type Tag = String;

/// The contents of the video or image that the library entry is associated with - what kind of footage does the video show?
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case", tag = "category")]
pub enum ContentDescription {
    /// Default value - value not selected by user yet.
    #[default]
    Unspecified,

    /// The video shows the song select screen, the entire playthrough of one song, and the end screen.
    GameplayNormal { game: Option<GameId> },

    /// The video or image shows only gameplay, and does not show the score screen at the end.
    GameplayOnly { game: Option<GameId> },

    /// The video or image shows only the results screen, and does not show the gameplay.
    ResultsScreen { game: Option<GameId> },

    /// The video or image depicts some part of the game, but the contents of the video or image don't belong to any other more specific category.
    GameGeneric { game: Option<GameId> },

    /// The contents of the video or image don't belong to any other category.
    Other { description: Option<String> },
}

/// The quality state of the proof file.
///
/// Videos that are "raw" can be transcoded and lossily compressed to save space.
///
/// The enum is ordered from best quality (least destructive) to worst quality (most destructive).
///
/// Naming of quality states and actions that change the quality state is based on the analogy of storing physical paper documents (well... at least I tried):
/// * The first state of a video is [`QualityState::Raw`] - this is a video file that has been taken straight from the recording software, without any additional processing.
/// * You can "preserve" a video to keep it in its [`QualityState::Raw`] state.
/// * You can "fold" a video and it will become a [`QualityState::Folded`] video.
///   A folded video is pretty much visually lossless, and it takes up a lot less space, just like a folded sheet of paper.
/// * You can "mess up" the video and it will become a [`QualityState::Messy`] video.
///   A messy video is compressed, but it is still not a pain to watch. It may have a lower resolution and reduced bitrate so it might not be great for editing, or for pausing or zooming in on, but it should still be fine to watch on it's own.
/// * You can "crumple" the video and it will become a [`QualityState::Crumpled`] video.
///   A crumpled video is visibly lossily compressed, but takes up a whole lot less space
/// * You can "shred" the video and it will become a [`QualityState::Shredded`] video.
///   A shredded video is compressed to a terrible quality, but it will take up a very small amount of space, usually under 5 MiB.
///
/// Additionally, you can also:
/// * "trash" the video - which means it won't be processed, and will be moved straight to the system trash, and
/// * "delete" the video - which means it will be `rm`'d from the filesystem entirely, without even going to trash.
///
/// These actions are traditionally applied to the "raw" video only, but in practice more destructive actions can also be used on already messy, folded or crumpled videos.
///
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QualityState {
    /// Default value - value not selected by user yet.
    #[default]
    Unspecified,

    /// Raw unprocessed recording or replay file or stream vod, which may or may not have been cut using `ffmpeg` with with `-c copy`, or LosslessCut. Largest file and best quality.
    /// Not recommended to store for a long time.
    Raw,

    /// Transcoded cut video, but visually lossless. Takes up a lot less space because it is transcoded after the initial recording on a slower encoding preset.
    /// Useful for PBs and first FCs.
    ///
    /// Example ffmpeg settings:
    /// - Codec: libx265
    /// - Resolution: 1080p
    /// - Framerate: 60 fps
    /// - Quality: crf=26
    /// - Preset: slow          // Note: `slower` is way too slow for commonly processing 1080p videos, taking about 1 hour per minute of video. it can however be useful at lower resolutions.
    ///
    /// Results: (input video: 1080p60, duration: 00:03:02.684)
    /// - Time: 00:33:48 (5.40 fps, 0,090x speed) -- 00:11:06 per 1 minute of video
    /// - File size: 116.03 MiB (5332.4kbits/s)   -- 38.11 MiB per 1 minute of video
    ///
    /// See also: [`Operation::CompressFoldVideo`](crate::hive::jobs::process_library_video::Operation::CompressFoldVideo)
    Folded,

    /// Transcoded cut video, lossy but still looking good and definitely watchable. Possibly in worse resolution. Should take up less than 20 MiB per minute of video.
    ///
    /// Example ffmpeg settings:
    /// - Codec: libx265
    /// - Resolution: 720p
    /// - Framerate: 60 fps
    /// - Quality: crf=29
    /// - Preset: slower
    ///
    /// Results: (input video: 1080p60, duration: 00:03:02.684)
    /// - Time: 01:08:37 (2.66 fps, 0,044x speed) -- 00:22:32 per 1 minute of video
    /// - File size: 44.53 MiB (2048.8kbits/s)    -- 14.63 MiB per 1 minute of video
    ///
    /// See also: [`Operation::CompressMessUpVideo`](crate::hive::jobs::process_library_video::Operation::CompressMessUpVideo)
    Messy,

    /// Transcoded cut video, in worse resolution but still readable quality. Has to take up less than 4 MiB per minute of video.
    /// Useful for non-PB performances that would've usually been thrown in the trash entirely.
    ///
    /// Example ffmpeg settings:
    /// - Codec: libx265
    /// - Resolution: 480p
    /// - Framerate: 60 fps
    /// - Quality: crf=35
    /// - Preset: slow          // Note: we don't really care about quality here anymore, and the filesize does not decrease much further if `slower` is used instead. this is good enough.
    ///
    /// Results: (input video: 1080p60, duration: 00:03:02.684)
    /// - Time: 00:06:21 (28.74 fps, 0.479x speed) -- 00:02:05 per 1 minute of video
    /// - File size: 12.35 MiB (3758.1kbits/s)     -- 4.06 MiB per 1 minute of video
    ///
    /// See also: [`Operation::CompressCrumpleVideo`](crate::hive::jobs::process_library_video::Operation::CompressCrumpleVideo)
    Crumpled,

    /// Transcoded cut video, with terrible bitrate and 360p. Takes up around 2 MiB per minute of video.
    /// Useful for unfinished performances or otherwise something that should be deleted usually, but may come in handy later (for example, for counting attempts).
    ///
    /// Example ffmpeg settings:
    /// - Codec: libx265
    /// - Resolution: 360p
    /// - Framerate: 60 fps
    /// - Quality: crf=38
    /// - Preset: slow          // Note: due to a large amount of files in this case, apart from the file size, we care more about transcoding speed than the actual quality of the video. this is good enough.
    ///
    /// Results: (input video: 1080p60, duration: 00:03:02.684)
    /// - Time: 00:03:30 (52.12 fps, 0.869x speed) -- 00:01:09 per 1 minute of video
    /// - File size: 5.17 MiB (240.6kbits/s)       -- 1.70 MiB per 1 minute of video
    ///
    /// See also: [`Operation::CompressShredVideo`](crate::hive::jobs::process_library_video::Operation::CompressShredVideo)
    Shredded,
}

/// Kind of the library entry - is it a proof of a performance or something else?
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LibraryEntryKind {
    /// Default value - value not selected by user yet.
    #[default]
    #[serde(alias = "unset")] // temp alias for migration while testing, can be removed later
    Unspecified,

    /// Video not showing a performance, unrelated to proof stuff but still in library for some reason.
    NotProof,

    /// Video showing a performance, but not yet possible to associate with a performance - the performance is not saveable in database for some reason. for example, one-finger-challenge FCs.
    Unsupported,

    /// Video showing a performance, but not yet associated with a performance.
    NotLinkedYet,

    /// Video showing a performance, associated with a performance or multiple performances.
    Linked,
}

pub type MediaMetadata = HashMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClothInfo {
    /// UUID of the cloth proof file.
    pub uuid: UuidString,

    /// Start point of the cut-out video within the cloth, in nanoseconds.
    pub start_point: Option<NsLocalTimestamp>,

    /// End point of the cut-out video within the cloth, in nanoseconds.
    pub end_point: Option<NsLocalTimestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guess<T> {
    prediction: T,
    confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction<T> {
    program_identifier: String,
    timestamp: NsTimestamp,
    guesses: Vec<Guess<T>>,
}

pub type AutomaticallyDetected<T> = Vec<Prediction<T>>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutomaticContentDetectionInformation {
    song_title: AutomaticallyDetected<String>,
    song_artist: AutomaticallyDetected<String>,
    song_id: AutomaticallyDetected<String>,
    player_name: AutomaticallyDetected<String>,
    instrument: AutomaticallyDetected<String>,
    difficulty: AutomaticallyDetected<String>,
    score: AutomaticallyDetected<f64>,
    note_streak: AutomaticallyDetected<u64>,
    note_hits: AutomaticallyDetected<u64>,
    notes_total: AutomaticallyDetected<u64>,
}

/// An entry in the library database, containing information about proof videos and images, and other files inside of the library.
///
/// Every unique file inside of the library should have exactly one library entry.
/// Old files, which have been deleted, moved, or transcoded into other files, should *not* have their entries removed from the library.
/// This is to preserve information about the source files for processed and cut files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    /// UUID of the library entry / proof.
    pub uuid: UuidString,

    /// SHA256 hash of the file.
    ///
    /// [`None`] for YouTube proofs (for now at least).
    pub sha256: Option<String>,

    /// Known library locations of the file. Updated on rescan.
    pub library_urls: Vec<StplUrl>,

    /// ID of this video file on YouTube
    pub youtube_id: Option<String>,

    /// Is the media file linked to any performance? Will it be linked to a performance in the future? Or is this not a video of a performance at all?
    pub entry_kind: LibraryEntryKind,

    /// Some information about the files on disk from `stat`.
    ///
    /// Since there may be multiple files on disk with the same sha256 hash and different file `stat`s, this is stored as a dictionary.
    /// Each file gets an entry.
    /// Note that even if a file may be present in `library_urls`, it doesn't have to be present here.
    pub file_stat: HashMap<StplUrl, FileStat>,

    /// Metadata inside of the media file (creation_date, android version, video/audio stream count, other similar metadata).
    /// The exact contents depends on the type of the file.
    ///
    /// Currently, this is not used, and the metadata will always be empty.
    pub metadata: Option<MediaMetadata>,

    /// Category of the media that this entry describes - is it a screenshot, a video from a camera, a mobile screen recording, something else?
    #[serde(alias = "category")] // temp alias for migration while testing, can be removed later
    pub media_category: MediaCategory,

    /// Content of the video - whether the video is showing gameplay, just the results, or something else. This field also contains information about the game being played.
    ///
    /// This field can be used by sorting and filtering systems to show relevant videos to the user.
    #[serde(default)]
    pub content_description: ContentDescription,

    /// Is this a full raw recording/stream vod, or is it cut already and shows only the relevant performance?
    ///
    /// Set this to [`None`] if it is unknown whether the video has been cut or not.
    pub cut: Option<bool>,

    /// Is the video raw, compressed, crumpled, or shredded?
    pub quality: QualityState,

    /// An entry UUID of the source media file that this file was cut out from. Files cut out from the same file are said to be "cut from the same cloth".
    ///
    /// Set this to [`None`] if the cloth is not known, or the file is not cut.
    pub cloth: Option<ClothInfo>,

    /// An entry UUID of the source media file that this file was processed from. Pre-processed files are "dry" and post-processed files are "wet".
    ///
    /// Set this to [`None`] if the dry file is not known, or the file is not processed.
    pub dry: Option<UuidString>,

    /// List of entry UUIDs of source media files used to create this media file. Montages are made of multiple clips for example.
    ///
    /// Set this to `Some(Vec::new())` if the clips are not known. Set this to [`None`] if this is not a montage.
    pub clips: Option<Vec<UuidString>>,

    /// Timestamp (in nanoseconds) of the real-life time at the start of this recording.
    ///
    /// Set this to [`None`] if this information is not known or is not applicable (montages).
    pub timestamp_start: Option<NsTimestamp>,

    /// Timestamp (in nanoseconds) of the real-life time at the end of this recording.
    ///
    /// Set this to [`None`] if this information is not known or is not applicable (montages).
    pub timestamp_end: Option<NsTimestamp>,

    /// Duration of the (video) file.
    ///
    /// This may or may not be the same as the difference between [`Self::timestamp_start`] and [`Self::timestamp_end`].
    /// Files that have fragments cut-out from the middle, files that are sped up or slowed down, and files resulting from a montage will not follow this rule.
    ///
    /// Set this to [`None`] if this information is not known.
    /// Set this to 0 for singular images/frames.
    pub duration: Option<NsDuration>,

    /// AutomaticContentDetectionInformation
    pub automatic_content_detection_information: Option<AutomaticContentDetectionInformation>,

    /// List of tags that are assigned to this library entry by the user.
    #[serde(default)]
    pub tags: HashSet<Tag>,

    /// User-added comment for this library entry.
    pub comment: Option<String>,

    /// Timestamp (in nanoseconds) of when this file was added/scanned into the library.
    pub timestamp_added: NsTimestamp,
}

impl Default for LibraryEntry {
    fn default() -> Self {
        Self {
            // Explicitly set custom values
            uuid: Uuid::now_v7().into(),
            timestamp_added: NsTimestamp::now(),

            // Default values for other fields
            youtube_id: None,
            sha256: None,
            library_urls: Vec::new(),
            entry_kind: LibraryEntryKind::default(),
            file_stat: HashMap::new(),
            metadata: None,
            media_category: MediaCategory::default(),
            content_description: ContentDescription::default(),
            cut: None,
            quality: QualityState::default(),
            cloth: None,
            dry: None,
            clips: None,
            tags: HashSet::new(),
            comment: None,
            timestamp_start: None,
            timestamp_end: None,
            duration: None,
            automatic_content_detection_information: None,
        }
    }
}

#[derive(Debug, Default)]
pub struct LibraryDatabase {
    pub entries: Vec<LibraryEntry>,
}

#[derive(Debug, Error)]
pub enum InsertError {
    #[error("proof is already in the database: {0}")]
    ExistsAlready(Uuid),
}

impl LibraryDatabase {
    pub const STANDARD_PATH_SEGMENTS: [&str; 2] = ["data", "library_database.jsonl"];

    pub fn path_within_shared_repo() -> &'static RelativePath {
        static CACHE: LazyLock<RelativePathBuf> = LazyLock::new(|| relative_path_from_segments(&LibraryDatabase::STANDARD_PATH_SEGMENTS));
        &CACHE
    }

    pub fn find_entry_by_uuid(&self, uuid: Uuid) -> Option<&LibraryEntry> {
        self.entries.iter().find(|x| x.uuid.0 == uuid)
    }

    pub fn find_entry_by_uuid_mut(&mut self, uuid: Uuid) -> Option<&mut LibraryEntry> {
        self.entries.iter_mut().find(|x| x.uuid.0 == uuid)
    }

    pub fn find_entry_by_sha256_hash(&self, sha256: &str) -> Option<&LibraryEntry> {
        self.entries.iter().find(|x| x.sha256.as_deref() == Some(sha256))
    }

    pub fn find_entry_by_sha256_hash_mut(&mut self, sha256: &str) -> Option<&mut LibraryEntry> {
        self.entries.iter_mut().find(|x| x.sha256.as_deref() == Some(sha256))
    }

    pub fn find_entry_by_youtube_id(&self, youtube_id: &str) -> Option<&LibraryEntry> {
        self.entries
            .iter()
            .find(|x| x.youtube_id.as_ref().is_some_and(|id| id == youtube_id))
    }

    pub fn insert_or_merge(&mut self, relative_path: &RelativePath, sha256: String, domain: LibraryDomain) -> (Uuid, bool) {
        let url = StplUrl::new(domain.clone(), Some(relative_path.to_string()));
        if let Some(existing_entry) = self.find_entry_by_sha256_hash_mut(&sha256) {
            existing_entry.library_urls.push(url);
            (existing_entry.uuid.0, true)
        } else {
            let new_library_entry = LibraryEntry {
                library_urls: vec![url],
                sha256: Some(sha256),
                ..Default::default()
            };
            let uuid = new_library_entry.uuid.0;
            self.entries.push(new_library_entry);
            (uuid, false)
        }
    }

    pub fn fetch_or_insert(&mut self, relative_path: &RelativePath, sha256: String, domain: LibraryDomain) -> (Uuid, bool) {
        let url = StplUrl::new(domain.clone(), Some(relative_path.to_string()));
        if let Some(existing_entry) = self.find_entry_by_sha256_hash_mut(&sha256) {
            (existing_entry.uuid.0, true)
        } else {
            let new_library_entry = LibraryEntry {
                library_urls: vec![url],
                sha256: Some(sha256),
                ..Default::default()
            };
            let uuid = new_library_entry.uuid.0;
            self.entries.push(new_library_entry);
            (uuid, false)
        }
    }

    pub fn insert(&mut self, entry: LibraryEntry) -> Result<Uuid, InsertError> {
        if let Some(existing_performance) = self.find_entry_by_uuid(entry.uuid.0) {
            return Err(InsertError::ExistsAlready(existing_performance.uuid.0));
        }

        let uuid = entry.uuid;
        self.entries.push(entry);
        Ok(uuid.0)
    }
}

impl FileLockableData for LibraryDatabase {
    fn _inner_read<F: FileEx + ?Sized>(file_ex: &F) -> file_ex::Result<Option<Self>> {
        file_ex.read_from_jsonlines().map(|x| x.map(|y| Self { entries: y }))
    }
    fn _inner_write<F: FileEx + ?Sized>(&self, file_ex: &F) -> file_ex::Result<()> {
        file_ex.write_as_jsonlines(&self.entries)
    }
}
