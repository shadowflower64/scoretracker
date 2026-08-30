//! Example job that waits for a specified amount of time.
use crate::hive::job::{AnyJob, Fail, Job, Success};
use crate::hive::worker::Worker;
use crate::util::timestamp::NsDuration;
use crate::{info, log_fn_name};
use serde::{Deserialize, Serialize};
use smol::Timer;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepJob {
    duration: NsDuration,
}

impl Job for SleepJob {
    async fn run(&self, _worker: Arc<Worker>) -> Result<Success, Fail> {
        log_fn_name!("job:sleep");
        info!("sleeping for {}", self.duration);
        Timer::after(self.duration.as_std_duration()).await;
        Ok(Success::Void)
    }
    fn into_any(self) -> AnyJob {
        AnyJob::Sleep(self)
    }
}
