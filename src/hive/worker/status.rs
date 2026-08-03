use crate::util::uuid::UuidString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct WorkerStatus {
    working: bool,
    current_task: Option<UuidString>,
}
