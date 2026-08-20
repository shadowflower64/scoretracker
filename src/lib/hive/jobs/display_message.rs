//! Example job that prints a specified message to the console.
use crate::hive::job::{AnyJob, Fail, Job, Success};
use crate::hive::worker::Worker;
use crate::{info, log_fn_name};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayMessageJob {
    message: String,
}

impl Job for DisplayMessageJob {
    async fn run(&self, _worker: Arc<Worker>) -> Result<Success, Fail> {
        log_fn_name!("job:display_message");
        info!("{}", self.message);
        Ok(Success::Void)
    }
    fn into_any(self) -> AnyJob {
        AnyJob::DisplayMessage(self)
    }
}
