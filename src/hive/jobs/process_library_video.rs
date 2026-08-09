use crate::hive::job::{AnyJob, Failure, Job, Success};
use crate::hive::worker::Worker;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingType {
    CompressImportantVideo,
    CompressCrumpleVideo,
    CompressShredVideo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessLibraryVideoJob {
    pub source_path: PathBuf,
    pub processing_type: ProcessingType,
    pub destination_path: PathBuf,
}

impl Job for ProcessLibraryVideoJob {
    async fn run(&self, _worker: Arc<Worker>) -> Result<Success, Failure> {
        todo!()
    }
    fn into_any(self) -> AnyJob {
        AnyJob::ProcessLibraryVideo(self)
    }
}
