use scoretracker::hive::worker::{Worker, WorkerStartError};

pub fn worker_main() -> Result<(), WorkerStartError> {
    Worker::start_default()
}
