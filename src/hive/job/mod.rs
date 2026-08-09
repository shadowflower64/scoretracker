//! One piece of work to do.
//!
//! A job is one action that has to be done by a [`crate::hive::worker`].
//! One worker may take on a job, and then report a success, or a failure.
pub mod cut_library_video;

use crate::data::library::database::{ClothInfo, LibraryEntry};
use crate::data::library::stpl_url::StplUrl;
use crate::hive::job::cut_library_video::CutLibraryVideoJob;
use crate::hive::worker::Worker;
use crate::util::filelocked::{ClosedFileLocked, FileLockableDataDefault};
use crate::util::uuid::UuidString;
use crate::{data::library::database::LibraryDatabase, info, log_fn_name};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{path::PathBuf, thread::sleep, time::Duration};
use thiserror::Error;

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
    #[error("panic while running a job: {0}")]
    Panic(String),
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
pub enum AnyJob {
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
    CutLibraryVideo(CutLibraryVideoJob),
    ProcessVideo {
        source_proof_uuid: UuidString,
        source_path: PathBuf,
        processing_type: ProcessingType,
        destination_path: PathBuf,
    },
}

pub trait Job {
    fn run(&self, worker: Arc<Worker>) -> impl Future<Output = Result<Success, Failure>>;
}

impl Job for AnyJob {
    async fn run(&self, worker: Arc<Worker>) -> Result<Success, Failure> {
        let worker_info = worker.data().lock().unwrap().info.clone();
        let config = worker.config();
        // Helper functions
        let _open_library_db_readwrite = || {
            LibraryDatabase::lock_and_read_or_default(config.library_database_path(), Some(&worker_info))
                .map_err(|e| Failure::LibraryError(e.to_string()))
        };
        let _reopen_library_db =
            |library_db: ClosedFileLocked<LibraryDatabase>| library_db.reopen().map_err(|e| Failure::LibraryError(e.to_string()));

        // TODO: ^^These helper functions are copied to CutLibraryVideoJob.
        // They should be placed somewhere else, presumably in the worker itself. it would make a lot of sense that way, no?

        match self {
            Self::DisplayMessage { message } => {
                log_fn_name!("job:display_message");
                info!("{}", message);
                Ok(Success::Void)
            }
            Self::Sleep { time_nanos } => {
                sleep(Duration::from_nanos(*time_nanos));
                Ok(Success::Void)
            }
            Self::DisplayMessageAndSleep { message, time_nanos } => {
                log_fn_name!("job:display_message_and_sleep");
                info!("{}", message);
                sleep(Duration::from_nanos(*time_nanos));
                Ok(Success::Void)
            }
            Self::CutLibraryVideo(data) => data.run(worker).await,
            Self::ProcessVideo { .. } => {
                todo!()
            }
        }
    }
}
