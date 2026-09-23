//! Library database file handling.
//!
//! A library database file is a file shared globally across libraries, that maps "proof UUIDs" to actual information and metadata about the proof.
//! Every entry in a library database file contains information about the SHA256 hash of the proof file, the type of the file (recording, screenshot etc.),
//! the modification timestamps of the file, the state of the file (is it linked to any score? is it uploaded?), as well as other information.
use crate::{
    data::{
        library::{
            automatic_content_detection_information::AutomaticContentDetectionInformation,
            cloth_info::ClothInfo,
            content_description::ContentDescription,
            entry_kind::LibraryEntryKind,
            file_stat::{FileStat, FileStats},
            media_category::MediaCategory,
            media_metadata::MediaMetadata,
            quality_state::QualityState,
            sha256::Sha256Hash,
            stpl_url::StplUrl,
            tag::Tags,
        },
        scoreboard::metadata::ArbitraryMetadata,
    },
    util::{
        timestamp::{NsDuration, NsTimestamp},
        uuid::UuidString,
    },
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;

pub type GameId = String;

/// An entry in the library database, containing information about proof videos and images, and other files inside of the library.
///
/// Every unique file inside of the library should have exactly one library entry.
/// Old files, which have been deleted, moved, or transcoded into other files, should *not* have their entries removed from the library.
/// This is to preserve information about the source files for processed and cut files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    /// UUID of the library entry / proof.
    pub proof_uuid: UuidString,

    /// SHA256 hash of the file.
    ///
    /// [`None`] for YouTube proofs (for now at least).
    pub sha256: Option<Sha256Hash>,

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
    pub file_stat: FileStats,

    /// Metadata inside of the media file (creation_date, android version, video/audio stream count, other similar metadata).
    /// The exact contents depends on the type of the file.
    ///
    /// Currently, this is not used, and the metadata will always be empty.
    pub media_metadata: Option<MediaMetadata>,

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
    ///
    /// Set this to [`None`] if the quality state has not been selected by the user yet.
    pub quality: Option<QualityState>,

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
    pub tags: Tags,

    /// Timestamp (in nanoseconds) of when this file was added/scanned into the library.
    pub timestamp_added: NsTimestamp,

    /// Arbitrary user-added metadata.
    pub metadata: ArbitraryMetadata,
}

impl LibraryEntry {
    pub fn update_stat(&mut self, url: StplUrl, path: impl AsRef<Path>) {
        self.file_stat.insert(url, FileStat::from_path(path));
    }

    pub fn from_postgres_row(row: &postgres::Row) -> Result<Self, postgres::Error> {
        Ok(Self {
            proof_uuid: row.try_get("proof_uuid")?,
            sha256: row.try_get("sha256")?,
            library_urls: row.try_get("library_urls")?,
            youtube_id: row.try_get("youtube_id")?,
            entry_kind: row.try_get("entry_kind")?,
            file_stat: row.try_get("file_stat")?,
            media_metadata: row.try_get("media_metadata")?,
            media_category: row.try_get("media_category")?,
            content_description: row.try_get("content_description")?,
            cut: row.try_get("cut")?,
            quality: row.try_get("quality")?,
            cloth: row.try_get("cloth")?,
            dry: row.try_get("dry")?,
            clips: row.try_get("clips")?,
            timestamp_start: row.try_get("timestamp_start")?,
            timestamp_end: row.try_get("timestamp_end")?,
            duration: row.try_get("duration")?,
            automatic_content_detection_information: row.try_get("automatic_content_detection_information")?,
            tags: row.try_get("tags")?,
            timestamp_added: row.try_get("timestamp_added")?,
            metadata: row.try_get("metadata")?,
        })
    }
}

impl Default for LibraryEntry {
    fn default() -> Self {
        Self {
            // Explicitly set custom values
            proof_uuid: Uuid::now_v7().into(),
            timestamp_added: NsTimestamp::now(),

            // Default values for other fields
            youtube_id: None,
            sha256: None,
            library_urls: Vec::new(),
            entry_kind: LibraryEntryKind::default(),
            file_stat: FileStats::new(),
            media_metadata: None,
            media_category: MediaCategory::default(),
            content_description: ContentDescription::default(),
            cut: None,
            quality: None,
            cloth: None,
            dry: None,
            clips: None,
            tags: Tags::new(),
            timestamp_start: None,
            timestamp_end: None,
            duration: None,
            automatic_content_detection_information: None,
            metadata: ArbitraryMetadata::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ProofInsertError {
    #[error("proof is already in the database: {0}")]
    ExistsAlready(Uuid),
}

// /// Returns a UUID for an existing database record without modification, or creates a new record, inserts it into the database, and returns the UUID for that.
// pub fn fetch_or_insert(&mut self, sha256: String, url: StplUrl) -> (Uuid, bool) {
//     if let Some(existing_entry) = self.find_entry_by_sha256_hash_mut(&sha256) {
//         (existing_entry.uuid.0, true)
//     } else {
//         let new_library_entry = LibraryEntry {
//             library_urls: vec![url],
//             sha256: Some(sha256),
//             ..Default::default()
//         };
//         let uuid = new_library_entry.uuid.0;
//         self.entries.push(new_library_entry);
//         (uuid, false)
//     }
// }

// /// Inserts an entry into the database, returning an error if a record with this UUID already exists.
// pub fn insert(&mut self, entry: LibraryEntry) -> Result<Uuid, InsertError> {
//     if let Some(existing_performance) = self.find_entry_by_uuid(entry.uuid.0) {
//         return Err(InsertError::ExistsAlready(existing_performance.uuid.0));
//     }

//     let uuid = entry.uuid;
//     self.entries.push(entry);
//     Ok(uuid.0)
// }
