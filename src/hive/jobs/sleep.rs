use crate::hive::job::{AnyJob, Failure, Job, Success};
use crate::hive::worker::Worker;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, thread::sleep, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepJob {
    time_nanos: u64,
}

impl Job for SleepJob {
    async fn run(&self, _worker: Arc<Worker>) -> Result<Success, Failure> {
        sleep(Duration::from_nanos(self.time_nanos));
        Ok(Success::Void)
    }
    fn into_any(self) -> AnyJob {
        AnyJob::Sleep(self)
    }
}
