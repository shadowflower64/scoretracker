use crate::util::filelocked::{FileLockableDataDefault, FileLocked};
use crate::util::uuid::UuidString;
use crate::{config::Config, hive::worker::WorkerInfo, info, library::database::LibraryDatabase, log_fn_name};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, thread::sleep, time::Duration};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingType {
    CompressImportantVideo,
    CompressCrumpleVideo,
    CompressShredVideo,
}

#[derive(Debug, Clone, Error, Deserialize, Serialize)]
#[serde(tag = "type", content = "details")]
pub enum Error {
    #[error("unknown error while running a job")]
    UnknownError,
    #[error("could not open library: {0}")]
    LibraryError(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "details")]
pub enum Success {
    Void,
    ProcessedVideo { dry: UuidString, wet: UuidString },
    CutVideo { cloth: UuidString, fragment: UuidString },
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
    CutVideo {
        source_proof_uuid: UuidString,
        source_path: PathBuf,
        cut_point_start_ms: Option<u64>,
        cut_point_end_ms: Option<u64>,
        destination_path: PathBuf,
    },
    ProcessVideo {
        source_proof_uuid: UuidString,
        source_path: PathBuf,
        processing_type: ProcessingType,
        destination_path: PathBuf,
    },
}

fn open_library_database(config: &Config, worker_info: Option<&WorkerInfo>) -> Result<FileLocked<LibraryDatabase>, Error> {
    LibraryDatabase::lock_and_read_or_default(config.library_database_path(), worker_info).map_err(|e| Error::LibraryError(e.to_string()))
}

impl Job {
    pub fn run(&self, config: &Config, worker_info: Option<&WorkerInfo>) -> Result<Success, Error> {
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
            Job::CutVideo { .. } => {
                todo!()
            }
            Job::ProcessVideo { source_proof_uuid, .. } => {
                let _library = open_library_database(config, worker_info)?;
                let _ = Success::CutVideo {
                    cloth: *source_proof_uuid,
                    fragment: Uuid::now_v7().into(),
                };
                todo!()
            }
        }
    }
}
