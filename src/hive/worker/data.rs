use crate::util::{timestamp::NsTimestamp, uuid::UuidString};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    /// Name of the worker.
    ///
    /// For standard scoretracker workers, this is generated with the [`super::Worker::make_name`] function.
    /// That function generates names in the format: `{random_name}-{pid}.scoretracker-worker.local`,
    /// where:
    /// * `random_name` is a lowercase name chosen using [`super::names::random_name`],
    /// * `pid` is the numeric ID of the worker process.
    // #[serde(alias = "name")]
    pub full_name: String,

    /// The "random name" part of full name of the worker.
    // #[serde(default)]
    pub short_name: String,

    /// Process ID of the worker process.
    pub pid: u32,

    /// Timestamp of the creation of the worker process
    pub birth_timestamp: NsTimestamp,

    /// Address for the listener for this worker. Can be used by other processes to communicate with this worker process.
    pub address: SocketAddr,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct WorkerStatus {
    pub working: bool,
    pub current_task: Option<UuidString>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TaskProgress {
    /// Is task fully done?
    pub done: bool,

    /// Number of the current stage, starting from 1, can go up to and including [`Self::total_stages`].
    pub current_stage_number: u32,

    /// Current stage name.
    pub current_stage: String,

    /// Total count of stages.
    pub total_stages: u64,

    /// How many subtasks have been done in this stage
    pub current_stage_progress: u64,

    /// How many subtasks are there to do in this stage
    pub current_stage_progress_max: u64,

    /// Progress message
    pub current_stage_progress_msg: String,
}

#[derive(Debug, Clone)]
pub struct WorkerData {
    /// Basic immutable information about the worker.
    pub info: WorkerInfo,

    /// Current status of the worker (is it working? idle? for how long?)
    pub status: WorkerStatus,

    /// Progress status of the currently executed task, or the task that was executed most recently.
    pub task_progress: Option<TaskProgress>,
}
