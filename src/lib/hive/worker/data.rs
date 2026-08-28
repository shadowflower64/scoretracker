use crate::hive::worker::UnwindSafeMutex;
use crate::util::{timestamp::NsTimestamp, uuid::UuidString};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    /// Name of the worker.
    ///
    /// For standard scoretracker workers, this is generated with the [`super::Worker::make_full_name`] function.
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

impl WorkerInfo {
    /// Returns a filename for this worker's log file, with the worker's birth date, name and pid.
    pub fn log_filename(&self) -> String {
        let birth_time: SystemTime = self
            .birth_timestamp
            .try_into()
            .expect("worker birth timestamp should be convertible to a SystemTime");
        let birth_datetime: DateTime<Local> = birth_time.into();
        let datetime = birth_datetime.format("%Y_%m_%d_%H_%M_%S");
        let short_name = self.short_name.to_lowercase();
        let pid = self.pid;
        format!("log_{datetime}_{short_name}-{pid}_worker.log")
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerState {
    /// Initial state.
    #[default]
    Initial,

    /// Currently working on a task.
    Working,

    /// Just finished working on a task.
    Finished,

    /// Idle, waiting for new tasks.
    Idle,

    /// Sleeping period inbetween task executions.
    Sleeping,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct WorkerStatus {
    pub state: WorkerState,
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

#[derive(Debug)]
pub struct WorkerData {
    /// Basic immutable information about the worker.
    pub info: WorkerInfo,

    /// Current status of the worker (is it working? idle? for how long?)
    pub status: UnwindSafeMutex<WorkerStatus>,

    /// Progress status of the currently executed task, or the task that was executed most recently.
    pub task_progress: UnwindSafeMutex<Option<TaskProgress>>,
}
