use scoretracker::hive::worker::{Worker, WorkerStartError};

pub fn worker_main() -> Result<(), WorkerStartError> {
    smol::block_on(async move { Worker::start_default().await })
}
