mod ipc;
pub mod names;
pub mod status;

use crate::config::Config;
use crate::hive::queue::{TaskNotFound, TaskQueue};
use crate::hive::task::{Task, TaskResult, TaskState};
use crate::hive::worker::ipc::start_listener_thread;
use crate::hive::worker::names::random_name;
use crate::hive::worker::status::WorkerStatus;
use crate::util::filelocked::{FileLockableDataDefault, FileLocked};
use crate::util::lockfile;
use crate::util::timestamp::NsTimestamp;
use crate::{error, info, log_fn_name, success};
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};
use std::{io, process};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot read from queue: {0}")]
    CannotReadQueue(lockfile::Error),
    #[error("cannot reopen queue: {0}")]
    CannotReopenQueue(lockfile::Error),
    #[error("cannot write to queue: {0}")]
    CannotWriteQueue(lockfile::Error),
    #[error("cannot update task in queue: {0}")]
    CannotUpdateTask(TaskNotFound),
    #[error("task not found: {0}")]
    TaskNotFound(Uuid),
    #[error("no tasks to do")]
    NoTopQueuedTask,
}

#[derive(Debug, Error)]
pub enum WorkerCreateError {
    #[error("configuration error: {0}")]
    ConfigError(#[from] lockfile::Error),
    #[error("could not bind tcp listener: {0}")]
    TcpListenerBindError(io::Error),
    #[error("could not get local address of tcp listener: {0}")]
    TcpListenerLocalAddrError(io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    pub name: String,
    pub pid: u32,
    pub birth_timestamp: NsTimestamp,
    pub address: SocketAddr,
}

#[derive(Debug)]
pub struct Worker {
    info: WorkerInfo,
    config: Config,
    worker_status: Arc<Mutex<WorkerStatus>>,
}

impl Worker {
    pub fn worker_info(&self) -> &WorkerInfo {
        &self.info
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn fetch_progress(&self) -> String {
        return "Fetch progress".to_owned(); // todo
    }

    pub fn new_with_listener(name: String, config: Config, listener: TcpListener) -> Result<Self, WorkerCreateError> {
        log_fn_name!("worker:new_with_listener");
        info!("creating worker with name: '{name}'");
        let address = listener.local_addr().map_err(WorkerCreateError::TcpListenerLocalAddrError)?;
        let worker_status = Arc::new(Mutex::new(WorkerStatus::default()));
        let worker = Worker {
            info: WorkerInfo {
                name,
                pid: process::id(),
                birth_timestamp: NsTimestamp::now(),
                address,
            },
            config,
            worker_status: worker_status.clone(),
        };
        let (worker_status_tx, worker_status_rx) = crossbeam_channel::unbounded();
        let (task_progress_tx, task_progress_rx) = crossbeam_channel::unbounded();
        start_listener_thread(listener, worker_status, worker_status_rx, task_progress_rx);
        Ok(worker)
    }

    pub fn new(name: String, config: Config) -> Result<Self, WorkerCreateError> {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(WorkerCreateError::TcpListenerBindError)?;
        Self::new_with_listener(name, config, listener)
    }

    pub fn new_default() -> Result<Self, WorkerCreateError> {
        let pid = process::id();
        let config = Config::load()?;
        let random_name = random_name().to_lowercase();
        Self::new(format!("{random_name}{pid}.scoretracker-worker.local"), config)
    }

    pub fn open_queue(&self) -> Result<FileLocked<TaskQueue>, Error> {
        TaskQueue::lock_and_read_or_default(self.config.task_queue_path(), Some(self.worker_info())).map_err(Error::CannotReadQueue)
    }

    /// Execute a task in the current thread.
    ///
    /// Please note that executing a task may take a long time.
    ///
    /// The task should be marked as "being worked on" before executing this method to prevent other processes from doing the same task.
    /// The queue file should be written to before calling this method.
    ///
    /// The result of the task should also be saved to the queue after this method finishes, so that no data is lost and the task is not done twice.
    async fn execute_task_body(&self, task: &mut Task) {
        log_fn_name!("worker:execute task");

        match task.job.run(&self.config, Some(self.worker_info())).await {
            Ok(success) => {
                success!("task finished successfully: uuid: {} results: {:#?}", task.uuid.0, success);
                task.state = TaskState::Done;
                task.result = Some(TaskResult::Success(success));
            }
            Err(error) => {
                error!("task failed: uuid: {} error: {:?}", task.uuid.0, error);
                task.state = TaskState::Failed;
                task.result = Some(TaskResult::Error(error));
            }
        }
        task.finish_timestamp = Some(NsTimestamp::now());
    }

    /// Execute a task from the queue in the current thread.
    ///
    /// Please note that executing a task may take a long time.
    ///
    /// This function will mark the task as being worked on and write to the [`TaskQueue`] file using [`lockfile`];
    /// only after marking the task in the queue will the task start being executed.
    /// After the task finishes, the results of the task are written automatically to the queue file.
    pub async fn execute_task<F: Fn(&mut FileLocked<TaskQueue>) -> Result<&mut Task, Error>>(
        &self,
        mut queue: FileLocked<TaskQueue>,
        task_getter: F,
    ) -> Result<FileLocked<TaskQueue>, Error> {
        log_fn_name!("worker:exec_task_safe");

        // Take on a task
        let task_to_do = task_getter(&mut queue)?;
        task_to_do.state = TaskState::Working;
        task_to_do.start_timestamp = Some(NsTimestamp::now());
        task_to_do.worker_info = Some(self.info.clone());
        // task_to_do.comment = Some(String::from("this job was started by scoretracker"));

        let mut task = task_to_do.clone();
        info!("taking on task with uuid: {}", task.uuid.0);

        // Drop file lock here to and let other processes access the queue
        let queue = queue.save_and_close().map_err(Error::CannotWriteQueue)?;

        // Do some task if there is something to do
        info!("starting task: {:?}", task);
        {
            let mut status = self.worker_status.lock().unwrap();
            status.working = true;
            status.current_task = Some(task.uuid);
        }
        self.execute_task_body(&mut task).await;
        {
            let mut status = self.worker_status.lock().unwrap();
            status.working = false;
            status.current_task = None;
        }

        // Update the queue file again to update the state of the task
        let mut queue = queue.reopen().map_err(Error::CannotReopenQueue)?;
        queue.update_task(task).map_err(Error::CannotUpdateTask)?;
        queue.save_to_file().map_err(Error::CannotWriteQueue)?;
        Ok(queue)
    }

    /// Execute a task from the queue in the current thread.
    ///
    /// Please note that executing a task may take a long time.
    ///
    /// This function uses [`Worker::execute_task`] with a simple getter function - see the documentation of [`Worker::execute_task`] for more information.
    pub async fn execute_task_with_uuid(&self, task_uuid: Uuid) -> Result<(), Error> {
        let queue = self.open_queue()?;
        self.execute_task(queue, |q| q.get_task_mut(task_uuid).ok_or(Error::TaskNotFound(task_uuid)))
            .await?;
        Ok(())
    }

    /// Take on the first task from the queue and execute it in the current thread.
    ///
    /// Please note that executing a task may take a long time.
    ///
    /// This function uses [`Worker::execute_task`] with a simple getter function - see the documentation of [`Worker::execute_task`] for more information.
    pub async fn take_on_task(&self) -> Result<(), Error> {
        let queue = self.open_queue()?;
        self.execute_task(queue, |q| q.top_queued_task_mut().ok_or(Error::NoTopQueuedTask))
            .await?;
        Ok(())
    }
}
