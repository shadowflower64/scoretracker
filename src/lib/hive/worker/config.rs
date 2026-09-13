use crate::hive::worker::{WorkerStartError, data::WorkerInfo, queue_connection::QueueConnection};
use crate::{config::toml::TomlConfig, hive::queue::TaskQueue, util::filelocked::FileLockableData};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub enum WorkerMode {
    /// Work on one task only, quit after finishing or failing to do the task. Quit immediately if there are no tasks to do.
    Single,

    /// Work on tasks continuously. If there are no tasks left, end process, do not idle in the background.
    Volatile,

    /// Work on tasks continuously. If there are no tasks left, stay idle in the background, waiting for new tasks to arrive.
    #[default]
    Persistent,
}

impl WorkerMode {
    pub fn quit_when_no_tasks(self) -> bool {
        match self {
            Self::Single | Self::Volatile => true,
            Self::Persistent => false,
        }
    }

    pub fn enable_loop(self) -> bool {
        match self {
            Self::Single => false,
            Self::Volatile | Self::Persistent => true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct WorkerConfig {
    /// How should the worker work?
    pub mode: WorkerMode,

    /// Username used to connect to the server
    pub family_name: String,

    /// Password used to connect to the server
    pub family_key: String,

    // Path to local queue file to use instead of a server connection.
    pub queue_file_path: Option<PathBuf>,

    /// How long should the worker sleep inbetween tasks.
    pub sleep_duration_seconds: f64,
}

impl TomlConfig for WorkerConfig {
    const STANDARD_FILENAME: &str = "worker.toml";
}

impl WorkerConfig {
    pub fn create_queue_connection(&self, worker_info: &WorkerInfo) -> Result<QueueConnection, WorkerStartError> {
        if let Some(queue_path) = &self.queue_file_path {
            let open = TaskQueue::lock_and_read(queue_path, Some(worker_info)).map_err(WorkerStartError::CannotOpenQueueFile)?;
            let closed = open.close_without_saving().map_err(WorkerStartError::CannotCloseQueueFile)?;
            QueueConnection::QueueFile(closed.into());
        }
        todo!();
    }
}
