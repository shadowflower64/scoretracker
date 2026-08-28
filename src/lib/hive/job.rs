//! One piece of work to do.
//!
//! A job is one action that has to be done by a [`crate::hive::worker`].
//! One worker may take on a job, and then report a success, or a failure.
use crate::data::library::database::{ClothInfo, LibraryEntry};
use crate::data::library::stpl_url::StplUrl;
use crate::ffmpeg::FFmpegError;
use crate::hive::jobs::cut_library_video::CutLibraryVideoJob;
use crate::hive::jobs::display_message::DisplayMessageJob;
use crate::hive::jobs::display_message_and_sleep::DisplayMessageAndSleepJob;
use crate::hive::jobs::process_library_video::ProcessLibraryVideoJob;
use crate::hive::jobs::sleep::SleepJob;
use crate::hive::worker::{self, Worker};
use crate::util::uuid::UuidString;
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

/// Successful result of a job.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "details")]
pub enum Success {
    Void,
    ProcessedVideo { dry: UuidString, wet: Option<UuidString> },
    CutVideo { cloth: ClothInfo, fragment: Option<UuidString> },
}

/// Failed result of a job.
#[derive(Debug, Clone, Error, Deserialize, Serialize)]
#[serde(tag = "type", content = "details")]
pub enum Fail {
    #[error("unknown error while running a job")]
    UnknownError,
    #[error("panic while running a job: {0}")]
    Panic(String),
    //
    #[error("ffmpeg process error: {0}")]
    FFmpegError(String),
    //
    #[error("could not open library database: {0}")]
    CannotOpenLibraryDatabase(String),
    #[error("could not read library database: {0}")]
    CannotReadLibraryDatabase(String),
    #[error("could not read library info: {0}")]
    CannotReadLibraryInfo(String),
    #[error("could not read library index: {0}")]
    CannotReadLibraryIndex(String),
    //
    #[error("could not find library entry with uuid: {0}")]
    EntryNotFound(UuidString),
    #[error("proof url is not present in the provided library entry: {expected_url}, {entry:?}")]
    FileUrlNotFoundInLibraryEntry { expected_url: StplUrl, entry: Box<LibraryEntry> },
    #[error("cannot find path to file witin library dir: library: {library_dir:?}; target file: {target_file_path:?}")]
    CannotFindPathWithinLibraryDir { library_dir: PathBuf, target_file_path: PathBuf },
    #[error(
        "file is not present in library index; perhaps the library needs to be rescanned? library: {library_dir:?}; internal path: {target_relpath:?}"
    )]
    FileNotInIndex {
        library_dir: PathBuf,
        target_relpath: RelativePathBuf,
    },
    #[error("cannot register file at {file_path:?} into library: {reason}")]
    CannotRegisterFileIntoLibrary { file_path: PathBuf, reason: String },
    #[error(
        "precondition check failed: uuid detected from path does not match precondition uuid ({read_proof_uuid} != {precondition_uuid}); path: {file_path:?}"
    )]
    PreconditionUuidDoesNotMatch {
        file_path: PathBuf,
        read_proof_uuid: UuidString,
        precondition_uuid: UuidString,
    },
    #[error("provided path is not part of any library: {path:?}")]
    PathNotInLibraryRepo { path: PathBuf },
    #[error("{message}")]
    Custom { message: String },
}

impl From<FFmpegError> for Fail {
    fn from(value: FFmpegError) -> Self {
        Self::FFmpegError(value.to_string())
    }
}

impl From<worker::WorkerError> for Fail {
    fn from(value: worker::WorkerError) -> Self {
        match value {
            worker::WorkerError::CannotOpenLibraryDatabase(e) => Self::CannotOpenLibraryDatabase(e.to_string()),
            worker::WorkerError::CannotReadLibraryDatabase(e) => Self::CannotReadLibraryDatabase(e.to_string()),
            worker::WorkerError::CannotReadLibraryInfo(e) => Self::CannotReadLibraryInfo(e.to_string()),
            worker::WorkerError::CannotReadLibraryIndex(e) => Self::CannotReadLibraryIndex(e.to_string()),
            worker::WorkerError::CannotOpenQueueFile(_)
            | worker::WorkerError::CannotReopenQueueFile(_)
            | worker::WorkerError::CannotCloseQueueFile(_)
            | worker::WorkerError::CannotUpdateTask(_)
            | worker::WorkerError::CannotWriteQueueFile(_)
            | worker::WorkerError::NoTopQueuedTask
            | worker::WorkerError::TaskNotFound(_) => unimplemented!(),
        }
    }
}

/// Enum containing variants for every job that a worker can do.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
#[serde(rename_all = "snake_case")]
pub enum AnyJob {
    Sleep(SleepJob),
    DisplayMessage(DisplayMessageJob),
    DisplayMessageAndSleep(DisplayMessageAndSleepJob),
    CutLibraryVideo(CutLibraryVideoJob),
    ProcessLibraryVideo(ProcessLibraryVideoJob),
}

/// Trait implemented by structures representing jobs.
pub trait Job {
    // Note to self: this is an Arc<Worker> because tasks may run separate threads and they may need to be able to share the worker pointer between threads.
    fn run(&self, worker: Arc<Worker>) -> impl Future<Output = Result<Success, Fail>>;
    fn into_any(self) -> AnyJob;
}

impl Job for AnyJob {
    async fn run(&self, worker: Arc<Worker>) -> Result<Success, Fail> {
        match self {
            Self::Sleep(job) => job.run(worker).await,
            Self::DisplayMessage(job) => job.run(worker).await,
            Self::DisplayMessageAndSleep(job) => job.run(worker).await,
            Self::CutLibraryVideo(job) => job.run(worker).await,
            Self::ProcessLibraryVideo(job) => job.run(worker).await,
        }
    }
    fn into_any(self) -> AnyJob {
        self
    }
}
