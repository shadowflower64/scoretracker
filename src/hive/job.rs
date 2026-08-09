//! One piece of work to do.
//!
//! A job is one action that has to be done by a [`crate::hive::worker`].
//! One worker may take on a job, and then report a success, or a failure.
use crate::data::library::database::{ClothInfo, LibraryEntry};
use crate::data::library::stpl_url::StplUrl;
use crate::hive::jobs::cut_library_video::CutLibraryVideoJob;
use crate::hive::jobs::display_message::DisplayMessageJob;
use crate::hive::jobs::display_message_and_sleep::DisplayMessageAndSleepJob;
use crate::hive::jobs::process_library_video::ProcessLibraryVideoJob;
use crate::hive::jobs::sleep::SleepJob;
use crate::hive::worker::{self, Worker};
use crate::util::uuid::UuidString;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "details")]
pub enum Success {
    Void,
    ProcessedVideo { dry: UuidString, wet: UuidString },
    CutVideo { cloth: ClothInfo, fragment: UuidString },
}

#[derive(Debug, Clone, Error, Deserialize, Serialize)]
#[serde(tag = "type", content = "details")]
pub enum Failure {
    #[error("unknown error while running a job")]
    UnknownError,
    #[error("panic while running a job: {0}")]
    Panic(String),
    #[error("could not open library database: {0}")]
    CannotOpenLibraryDatabase(String),
    #[error("could not read library database: {0}")]
    CannotReadLibraryDatabase(String),
    #[error("could not read library info: {0}")]
    CannotReadLibraryInfo(String),
    #[error("could not find library entry with uuid: {0}")]
    EntryNotFound(UuidString),
    #[error("proof url is not present in the provided library entry: {expected_url}, {entry:?}")]
    FileUrlNotFoundInEntry { expected_url: StplUrl, entry: Box<LibraryEntry> },
    #[error("cannot register file at {file_path:?} into library: {reason}")]
    CannotRegisterFileIntoLibrary { file_path: PathBuf, reason: String },
    #[error("{message}")]
    Custom { message: String },
}

impl From<worker::Error> for Failure {
    fn from(value: worker::Error) -> Self {
        match value {
            worker::Error::CannotOpenLibraryDatabase(e) => Self::CannotOpenLibraryDatabase(e.to_string()),
            worker::Error::CannotReadLibraryDatabase(e) => Self::CannotReadLibraryDatabase(e.to_string()),
            worker::Error::CannotReadLibraryInfo(e) => Self::CannotReadLibraryInfo(e.to_string()),
            worker::Error::CannotOpenQueue(_)
            | worker::Error::CannotReopenQueue(_)
            | worker::Error::CannotUpdateTask(_)
            | worker::Error::CannotWriteQueue(_)
            | worker::Error::NoTopQueuedTask
            | worker::Error::TaskNotFound(_) => unimplemented!(),
        }
    }
}

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

pub trait Job {
    fn run(&self, worker: Arc<Worker>) -> impl Future<Output = Result<Success, Failure>>;
    fn into_any(self) -> AnyJob;
}

impl Job for AnyJob {
    async fn run(&self, worker: Arc<Worker>) -> Result<Success, Failure> {
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
