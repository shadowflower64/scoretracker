use crate::hive::job::{Failure, Job, Success};
use crate::hive::worker::Worker;
use crate::{info, log_fn_name};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayMessageJob {
    message: String,
}

impl Job for DisplayMessageJob {
    async fn run(&self, _worker: Arc<Worker>) -> Result<Success, Failure> {
        log_fn_name!("job:display_message");
        info!("{}", self.message);
        Ok(Success::Void)
    }
}
