use crate::util::uuid::UuidString;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct WorkerStatus {
    pub working: bool,
    pub current_task: Option<UuidString>,
}
