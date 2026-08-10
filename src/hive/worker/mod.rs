pub mod data;
pub mod ipc;
pub mod names;
pub mod status;

use crate::config::Config;
use crate::data::library::database::LibraryDatabase;
use crate::data::library::index::LibraryIndex;
use crate::data::library::info::LibraryInfo;
use crate::hive::job::{self, Job};
use crate::hive::queue::{TaskNotFound, TaskQueue};
use crate::hive::task::{Task, TaskResult, TaskState};
use crate::hive::worker::data::{TaskProgress, WorkerData, WorkerInfo, WorkerStatus};
use crate::hive::worker::ipc::start_listener_thread;
use crate::hive::worker::names::random_name;
use crate::util::dirs::log_dir;
use crate::util::filelocked::{ClosedFileLocked, FileLockableData, FileLockableDataDefault, FileLocked};
use crate::util::log::{LogError, open_log_file};
use crate::util::timestamp::NsTimestamp;
use crate::util::{file_ex, lockfile};
use crate::{error, info, log_fn_name, success, warn};
use crossbeam_channel::Sender;
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{io, panic, process};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot read lock from queue: {0}")]
    CannotOpenQueue(lockfile::Error),
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
    #[error("cannot read lock library database: {0}")]
    CannotOpenLibraryDatabase(lockfile::Error),
    #[error("cannot read library database: {0}")]
    CannotReadLibraryDatabase(file_ex::Error),
    #[error("cannot read library info: {0}")]
    CannotReadLibraryInfo(file_ex::Error),
    #[error("cannot read library index: {0}")]
    CannotReadLibraryIndex(file_ex::Error),
}

#[derive(Debug, Error)]
pub enum WorkerCreateError {
    #[error("configuration error: {0}")]
    ConfigError(#[from] file_ex::Error),
    #[error("cannot open log file: {0}")]
    LogError(LogError),
    #[error("could not bind tcp listener: {0}")]
    TcpListenerBindError(io::Error),
    #[error("could not get local address of tcp listener: {0}")]
    TcpListenerLocalAddrError(io::Error),
}

#[derive(Debug)]
pub struct Worker {
    config: Config,

    /// All data that is related to this worker.
    ///
    /// This structure is shareable across threads.
    /// It contains:
    /// * [`WorkerInfo`], an immutable structure containing basic info about the worker;
    /// * [`WorkerStatus`], which holds the current state of the worker (is it paused, running, etc.);
    /// * [`TaskProgress`], which contains information about progress on the current task.
    data: Arc<Mutex<WorkerData>>,
    worker_status_tx: Sender<WorkerStatus>,
    task_progress_tx: Sender<TaskProgress>,
}

impl Worker {
    pub fn data(&self) -> &Arc<Mutex<WorkerData>> {
        &self.data
    }

    pub fn info_cloned(&self) -> WorkerInfo {
        self.data().lock().unwrap().info.clone()
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn open_queue(&self) -> Result<FileLocked<TaskQueue>, Error> {
        TaskQueue::lock_and_read_or_default(self.config.task_queue_path(), Some(&self.info_cloned())).map_err(Error::CannotOpenQueue)
    }

    pub fn open_library_db(&self) -> Result<FileLocked<LibraryDatabase>, Error> {
        LibraryDatabase::lock_and_read_or_default(self.config.library_database_path(), Some(&self.info_cloned()))
            .map_err(Error::CannotOpenLibraryDatabase)
    }

    pub fn read_library_db(&self) -> Result<LibraryDatabase, Error> {
        LibraryDatabase::read_without_locking_or_default(self.config.library_database_path()).map_err(Error::CannotReadLibraryDatabase)
    }

    pub fn reopen_library_db(&self, library_db: ClosedFileLocked<LibraryDatabase>) -> Result<FileLocked<LibraryDatabase>, Error> {
        library_db.reopen().map_err(Error::CannotOpenLibraryDatabase)
    }

    pub fn read_library_info(&self, library_dir: &Path) -> Result<LibraryInfo, Error> {
        LibraryInfo::read_without_locking(library_dir.join(LibraryInfo::STANDARD_FILENAME)).map_err(Error::CannotReadLibraryInfo)
    }

    pub fn read_library_index(&self, library_dir: &Path) -> Result<LibraryIndex, Error> {
        LibraryIndex::read_without_locking(library_dir.join(LibraryIndex::STANDARD_FILENAME)).map_err(Error::CannotReadLibraryIndex)
    }

    pub fn fetch_progress(&self) -> Option<TaskProgress> {
        self.data.lock().unwrap().task_progress.clone()
    }

    pub fn update_worker_status(&self, status: WorkerStatus) {
        log_fn_name!("update_worker_status");
        self.data.lock().unwrap().status = status.clone();
        let _ = self
            .worker_status_tx
            .send(status)
            .inspect_err(|e| warn!("worker_status_tx channel disconnected: {e:?}"));
    }

    pub fn update_task_progress(&self, task_progress: TaskProgress) {
        log_fn_name!("update_task_progress");
        self.data.lock().unwrap().task_progress = Some(task_progress.clone());
        let _ = self
            .task_progress_tx
            .send(task_progress)
            .inspect_err(|e| warn!("task_progress_tx channel disconnected: {e:?}"));
    }

    pub fn update_task_progress_very_simple(&self, message: String) {
        let task_progress = TaskProgress {
            done: false,
            current_stage_number: 1,
            current_stage: "Unknown stage".to_owned(),
            total_stages: 1,
            current_stage_progress: 0,
            current_stage_progress_max: 1,
            current_stage_progress_msg: message,
        };
        self.update_task_progress(task_progress);
    }

    pub fn new_with_listener(short_name: String, config: Config, listener: TcpListener) -> Result<Self, WorkerCreateError> {
        log_fn_name!("worker:new_with_listener");

        let pid = process::id();
        let full_name = Self::make_full_name(&short_name, pid);
        info!("creating worker with name: '{full_name}'");

        let address = listener.local_addr().map_err(WorkerCreateError::TcpListenerLocalAddrError)?;
        let info = WorkerInfo {
            full_name,
            short_name,
            pid,
            birth_timestamp: NsTimestamp::now(),
            address,
        };
        open_log_file(&log_dir().join(info.log_filename())).map_err(WorkerCreateError::LogError)?;

        let (worker_status_tx, worker_status_rx) = crossbeam_channel::unbounded();
        let (task_progress_tx, task_progress_rx) = crossbeam_channel::unbounded();
        let worker = Worker {
            config,
            data: Arc::new(Mutex::new(WorkerData {
                info,
                task_progress: None,
                status: WorkerStatus::default(),
            })),
            worker_status_tx,
            task_progress_tx,
        };
        start_listener_thread(listener, Arc::clone(worker.data()), worker_status_rx, task_progress_rx);
        Ok(worker)
    }

    pub fn new(short_name: String, config: Config) -> Result<Self, WorkerCreateError> {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(WorkerCreateError::TcpListenerBindError)?;
        Self::new_with_listener(short_name, config, listener)
    }

    pub fn make_full_name(short_name: &str, pid: u32) -> String {
        format!("{}-{pid}.scoretracker-worker.local", short_name.to_lowercase())
    }

    pub fn new_default() -> Result<Self, WorkerCreateError> {
        let config = Config::load()?;
        Self::new(random_name().to_owned(), config)
    }

    /// Execute a task in the current thread.
    ///
    /// Please note that executing a task may take a long time.
    ///
    /// The task should be marked as "being worked on" before executing this method to prevent other processes from doing the same task.
    /// The queue file should be written to before calling this method.
    ///
    /// The result of the task should also be saved to the queue after this method finishes, so that no data is lost and the task is not done twice.
    async fn execute_task_body(self: Arc<Self>, task: &mut Task) {
        log_fn_name!("worker:execute task");

        let result = panic::catch_unwind(|| smol::block_on(task.job.run(self)));
        match result {
            Ok(no_panic) => match no_panic {
                Ok(success) => {
                    success!("task finished successfully: uuid: {}, results: {:#?}", task.uuid.0, success);
                    task.state = TaskState::Done;
                    task.result = Some(TaskResult::Success(success));
                }
                Err(error) => {
                    error!("task failed: uuid: {}, reason: {} ({:?})", task.uuid.0, error, error);
                    task.state = TaskState::Failed;
                    task.result = Some(TaskResult::Error(error));
                }
            },
            Err(panic) => {
                let disp_panic = if let Some(a) = panic.downcast_ref::<String>() {
                    a.to_owned()
                } else if let Some(a) = panic.downcast_ref::<&str>() {
                    a.to_string()
                } else {
                    "<unknown>".to_string()
                };
                error!("task panicked: uuid: {}, reason: {disp_panic}", task.uuid.0);
                task.state = TaskState::Failed;
                task.result = Some(TaskResult::Error(job::Fail::Panic(disp_panic)));
                panic::resume_unwind(panic)
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
        self: Arc<Self>,
        mut queue: FileLocked<TaskQueue>,
        task_getter: F,
    ) -> Result<FileLocked<TaskQueue>, Error> {
        log_fn_name!("worker:exec_task_safe");

        // Take on a task
        let task_to_do = task_getter(&mut queue)?;
        task_to_do.state = TaskState::Working;
        task_to_do.start_timestamp = Some(NsTimestamp::now());
        task_to_do.worker_info = Some(self.data().lock().unwrap().info.clone());
        // task_to_do.comment = Some(String::from("this job was started by scoretracker"));

        let mut task = task_to_do.clone();
        info!("taking on task with uuid: {}", task.uuid.0);

        // Drop file lock here to and let other processes access the queue
        let queue = queue.save_and_close().map_err(Error::CannotWriteQueue)?;

        // Do some task if there is something to do
        info!("starting task: {:?}", task);
        {
            let mut data = self.data().lock().unwrap();
            data.status.working = true;
            data.status.current_task = Some(task.uuid);
        }
        self.clone().execute_task_body(&mut task).await;
        {
            let mut data = self.data().lock().unwrap();
            data.status.working = false;
            data.status.current_task = None;
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
    pub async fn execute_task_with_uuid(self: Arc<Self>, task_uuid: Uuid) -> Result<(), Error> {
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
    pub async fn take_on_task(self: Arc<Self>) -> Result<(), Error> {
        let queue = self.open_queue()?;
        self.execute_task(queue, |q| q.top_queued_task_mut().ok_or(Error::NoTopQueuedTask))
            .await?;
        Ok(())
    }
}
