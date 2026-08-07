//! One piece of work to do.
//!
//! A job is one action that has to be done by a [`crate::hive::worker`].
//! One worker may take on a job, and then report a success, or a failure.
use crate::data::library::database::{ClothInfo, LibraryEntry};
use crate::data::library::index::LibraryIndex;
use crate::data::library::scan_register_added_file;
use crate::data::library::stpl_url::StplUrl;
use crate::ffmpeg::ffmpeg_cut_video_streamcopy;
use crate::hive::worker::data::WorkerInfo;
use crate::util::filelocked::{ClosedFileLocked, FileLockableData, FileLockableDataDefault};
use crate::util::timestamp::NsLocalTimestamp;
use crate::util::uuid::UuidString;
use crate::{config::Config, data::library::database::LibraryDatabase, info, log_fn_name};
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{path::PathBuf, thread::sleep, time::Duration};
use thiserror::Error;
use uuid::Uuid;

fn create_stpl_url_to_file(_library_dir: &Path, _file_path: &Path) -> StplUrl {
    todo!()
}
fn create_internal_path_to_file(_library_dir: &Path, _file_path: &Path) -> RelativePathBuf {
    todo!()
}
fn get_library_dir_of_path(_path: &Path) -> Option<PathBuf> {
    todo!()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingType {
    CompressImportantVideo,
    CompressCrumpleVideo,
    CompressShredVideo,
}

#[derive(Debug, Clone, Error, Deserialize, Serialize)]
#[serde(tag = "type", content = "details")]
pub enum Failure {
    #[error("unknown error while running a job")]
    UnknownError,
    #[error("could not open library: {0}")]
    LibraryError(String),
    #[error("could not find library entry with uuid: {0}")]
    EntryNotFound(UuidString),
    #[error("proof url is not present in the provided library entry: {expected_url}, {entry:?}")]
    FileUrlNotFoundInEntry { expected_url: StplUrl, entry: Box<LibraryEntry> },
    #[error("cannot register file at {file_path:?} into library: {reason}")]
    CannotRegisterFileIntoLibrary { file_path: PathBuf, reason: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "details")]
pub enum Success {
    Void,
    ProcessedVideo { dry: UuidString, wet: UuidString },
    CutVideo { cloth: ClothInfo, fragment: UuidString },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
#[serde(rename_all = "snake_case")]
pub enum Job {
    Sleep {
        time_nanos: u64,
    },
    DisplayMessage {
        message: String,
    },
    DisplayMessageAndSleep {
        message: String,
        time_nanos: u64,
    },
    CutLibraryVideo {
        source_path: PathBuf,
        cut_start_point: Option<NsLocalTimestamp>,
        cut_end_point: Option<NsLocalTimestamp>,
        destination_path: PathBuf,
    },
    ProcessVideo {
        source_proof_uuid: UuidString,
        source_path: PathBuf,
        processing_type: ProcessingType,
        destination_path: PathBuf,
    },
}

impl Job {
    pub async fn run(&self, config: &Config, worker_info: Option<&WorkerInfo>) -> Result<Success, Failure> {
        // Helper functions
        let _open_library_db_readwrite = || {
            LibraryDatabase::lock_and_read_or_default(config.library_database_path(), worker_info)
                .map_err(|e| Failure::LibraryError(e.to_string()))
        };
        let open_library_db_readonly = || {
            LibraryDatabase::read_without_locking_or_default(config.library_database_path())
                .map_err(|e| Failure::LibraryError(e.to_string()))
        };
        let _reopen_library_db =
            |library_db: ClosedFileLocked<LibraryDatabase>| library_db.reopen().map_err(|e| Failure::LibraryError(e.to_string()));

        match self {
            Job::DisplayMessage { message } => {
                log_fn_name!("job:display_message");
                info!("{}", message);
                Ok(Success::Void)
            }
            Job::Sleep { time_nanos } => {
                sleep(Duration::from_nanos(*time_nanos));
                Ok(Success::Void)
            }
            Job::DisplayMessageAndSleep { message, time_nanos } => {
                log_fn_name!("job:display_message_and_sleep");
                info!("{}", message);
                sleep(Duration::from_nanos(*time_nanos));
                Ok(Success::Void)
            }
            Job::CutLibraryVideo {
                source_path,
                cut_start_point,
                cut_end_point,
                destination_path,
            } => {
                // Fetch the source proof UUID from the local index
                let source_library_dir = get_library_dir_of_path(source_path).expect("todo");
                let source_library_index =
                    LibraryIndex::read_without_locking(source_library_dir.join(LibraryIndex::STANDARD_FILENAME)).expect("todo");

                let internal_source_path = create_internal_path_to_file(&source_library_dir, source_path);
                let source_proof_uuid = source_library_index.files.get(&internal_source_path).expect("todo");

                // Get source proof entry from the database
                let library_db = open_library_db_readonly()?;
                let proof_entry = library_db
                    .find_entry_by_uuid(*source_proof_uuid)
                    .ok_or(Failure::EntryNotFound(*source_proof_uuid))?
                    .to_owned();
                drop(library_db);

                // Sanity check - proof entry with the given UUID should contain the given URL.
                // If it doesn't, that means the library needs to be rescanned or the request was invalid.
                let file_url = create_stpl_url_to_file(&source_library_dir, source_path);
                if !proof_entry.library_urls.contains(&file_url) {
                    return Err(Failure::FileUrlNotFoundInEntry {
                        expected_url: file_url,
                        entry: Box::new(proof_entry.to_owned()),
                    });
                }

                // Launch ffmpeg to cut the video losslessly
                ffmpeg_cut_video_streamcopy(
                    source_path,
                    destination_path,
                    cut_start_point.map(|x| x.as_millis() as u64),
                    cut_end_point.map(|x| x.as_millis() as u64),
                )
                .await;

                // Get library info of the destination file
                let destination_library_dir = get_library_dir_of_path(source_path).unwrap(); // TODO: error handling

                // Register new file to index and to database
                scan_register_added_file(
                    &destination_library_dir,
                    &config.library_database_path(),
                    destination_path,
                    worker_info,
                )
                .map_err(|e| Failure::CannotRegisterFileIntoLibrary {
                    file_path: destination_path.to_owned(),
                    reason: e.to_string(),
                })?;

                Ok(Success::CutVideo {
                    cloth: ClothInfo {
                        uuid: *source_proof_uuid,
                        start_point: *cut_start_point,
                        end_point: *cut_end_point,
                    },
                    fragment: Uuid::now_v7().into(),
                })
            }
            Job::ProcessVideo { .. } => {
                todo!()
            }
        }
    }
}
