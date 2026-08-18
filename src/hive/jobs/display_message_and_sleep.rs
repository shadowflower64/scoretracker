//! Example job that prints a specified message to the console and then waits for a specified amount of time.
use crate::hive::job::{AnyJob, Fail, Job, Success};
use crate::hive::worker::Worker;
use crate::{info, log_fn_name};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayMessageAndSleepJob {
    pub message: String,
    pub time_nanos: u64,
}

impl Job for DisplayMessageAndSleepJob {
    async fn run(&self, _worker: Arc<Worker>) -> Result<Success, Fail> {
        log_fn_name!("job:display_message_and_sleep");
        info!("{}", self.message);
        sleep(Duration::from_nanos(self.time_nanos));
        Ok(Success::Void)
    }
    fn into_any(self) -> AnyJob {
        AnyJob::DisplayMessageAndSleep(self)
    }
}
