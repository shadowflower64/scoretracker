//! Example job that waits for a specified amount of time.
use crate::hive::job::{AnyJob, Fail, Job, Success};
use crate::hive::worker::Worker;
use crate::util::timestamp::NsDuration;
use crate::{info, log_fn_name};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, thread::sleep};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepJob {
    duration: NsDuration,
}

impl Job for SleepJob {
    async fn run(&self, _worker: Arc<Worker>) -> Result<Success, Fail> {
        log_fn_name!("job:sleep");
        info!("sleeping for {}", self.duration);
        sleep(
            self.duration
                .try_into()
                .expect("NsDuration should be convertible into a std::time::Duration"),
        );
        Ok(Success::Void)
    }
    fn into_any(self) -> AnyJob {
        AnyJob::Sleep(self)
    }
}
