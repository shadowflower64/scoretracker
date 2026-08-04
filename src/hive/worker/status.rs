use crate::util::uuid::UuidString;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct WorkerStatus {
    pub working: bool,
    pub current_task: Option<UuidString>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TaskProgress {
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
