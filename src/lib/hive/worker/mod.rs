//! Module containing code for hive worker processes.
pub mod config;
pub mod data;
pub mod ipc;
pub mod names;
pub mod queue_connection;
pub mod status;
pub mod ws;

use crate::config::library_tab::{InternalLibraryConnections, LibraryAccessPath, LibraryTab};
use crate::config::toml::{TomlConfig, TomlConfigError};
use crate::data::library::database::LibraryEntry;
use crate::data::library::index::LibraryIndex;
use crate::data::library::info::LibraryInfo;
use crate::data::library::stpl_url::{LibraryDomain, StplUrl};
use crate::hive::job::{self, Fail, Job, Success};
use crate::hive::queue::TaskNotFound;
use crate::hive::task::{Task, TaskResult};
use crate::hive::worker::config::{WorkerConfig, WorkerMode};
use crate::hive::worker::data::{TaskProgress, WorkerData, WorkerInfo, WorkerState, WorkerStatus};
use crate::hive::worker::ipc::start_listener_thread;
use crate::hive::worker::queue_connection::QueueConnection;
use crate::hive::worker::ws::start_server_connection_thread;
use crate::util::dirs::log_dir;
use crate::util::filelocked::FileLockableData;
use crate::util::log::{LogError, open_log_file};
use crate::util::timestamp::NsTimestamp;
use crate::util::{file_ex, lockfile};
use crate::{error, info, log_fn_name, success, warn};
use crossbeam_channel::Sender;
use function_name::named;
use smol::lock::Mutex;
use std::borrow::Cow;
use std::net::TcpListener;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use std::{io, panic, process, thread};
use thiserror::Error;
use uuid::Uuid;

/// Error while performing worker functions.
#[derive(Debug, Error)]
pub enum WorkerError {
    #[error("cannot acquire lock for queue file: {0}")]
    CannotOpenQueueFile(lockfile::Error),
    #[error("cannot close lock for queue file: {0}")]
    CannotCloseQueueFile(lockfile::Error),
    #[error("cannot reopen queue file: {0}")]
    CannotReopenQueueFile(lockfile::Error),
    #[error("cannot write to queue file: {0}")]
    CannotWriteQueueFile(lockfile::Error),
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

/// Error while starting a worker process.
#[derive(Debug, Error)]
pub enum WorkerStartError {
    #[error("configuration error: {0}")]
    ConfigError(#[from] TomlConfigError),
    #[error("cannot open log file: {0}")]
    LogError(#[from] LogError),
    #[error("could not bind tcp listener: {0}")]
    TcpListenerBindError(io::Error),
    #[error("could not get local address of tcp listener: {0}")]
    TcpListenerLocalAddrError(io::Error),
    #[error("could not open queue file: {0}")]
    CannotOpenQueueFile(lockfile::Error),
    #[error("could not close queue file: {0}")]
    CannotCloseQueueFile(lockfile::Error),
}

// TODO: safety LOL!
type UnwindSafeMutex<T> = AssertUnwindSafe<Mutex<T>>;

/// Worker data structure, containing a config, some mutable data, and channels for sending updates to clients connected via TCP.
#[derive(Debug)]
pub struct Worker {
    config: WorkerConfig,
    data: Arc<WorkerData>,
    queue_connection: UnwindSafeMutex<QueueConnection>,
    internal_libraries: InternalLibraryConnections,
    worker_status_tx: Sender<WorkerStatus>,
    task_progress_tx: Sender<TaskProgress>,
}

impl Worker {
    /// All data that is related to this worker.
    ///
    /// This structure is shareable across threads.
    /// It contains:
    /// * [`WorkerInfo`], an immutable structure containing basic info about the worker;
    /// * [`WorkerStatus`], which holds the current state of the worker (is it paused, running, etc.);
    /// * [`TaskProgress`], which contains information about progress on the current task.
    pub fn data(&self) -> &Arc<WorkerData> {
        &self.data
    }

    /// A short way of fetching the [`WorkerInfo`] structure living inside the worker data.
    pub fn info(&self) -> &WorkerInfo {
        &self.data().info
    }

    /// Worker configuration
    pub fn config(&self) -> &WorkerConfig {
        &self.config
    }

    pub fn read_library_info(&self, library_dir: &Path) -> Result<LibraryInfo, WorkerError> {
        LibraryInfo::read_without_locking(library_dir.join(LibraryInfo::STANDARD_FILENAME)).map_err(WorkerError::CannotReadLibraryInfo)
    }

    pub fn read_library_index(&self, library_dir: &Path) -> Result<LibraryIndex, WorkerError> {
        LibraryIndex::read_without_locking(library_dir.join(LibraryIndex::STANDARD_FILENAME)).map_err(WorkerError::CannotReadLibraryIndex)
    }

    pub async fn fetch_progress(&self) -> Option<TaskProgress> {
        self.data.task_progress.lock().await.clone()
    }

    /// Save the status to internal data and also send a worker status update message to clients connected via TCP.
    #[named]
    pub async fn update_worker_status(&self, status: WorkerStatus) {
        log_fn_name!(auto);
        *self.data.status.lock().await = status.clone();
        let _ = self
            .worker_status_tx
            .send(status)
            .inspect_err(|e| warn!("worker_status_tx channel disconnected: {e:?}"));
    }

    /// Save the progress to internal data and also send a task progress update message to clients connected via TCP.
    #[named]
    pub async fn update_task_progress(&self, task_progress: TaskProgress) {
        log_fn_name!(auto);
        *self.data.task_progress.lock().await = Some(task_progress.clone());
        let _ = self
            .task_progress_tx
            .send(task_progress)
            .inspect_err(|e| warn!("task_progress_tx channel disconnected: {e:?}"));
    }

    /// A very over-simplified way of updating task progress using just a string.
    // TODO: this shouldn't really be used in final code
    // #[deprecated]
    pub async fn update_task_progress_very_simple(&self, message: String) {
        let task_progress = TaskProgress {
            done: false,
            current_stage_number: 1,
            current_stage: "Unknown stage".to_owned(),
            total_stages: 1,
            current_stage_progress: 0,
            current_stage_progress_max: 1,
            current_stage_progress_msg: message,
        };
        self.update_task_progress(task_progress).await
    }

    /// Create a full worker name from a short name (usually chosen randomly from [`names`]) and a process ID (pid) number.
    pub fn make_full_name(short_name: &str, pid: u32) -> String {
        format!("{}-{pid}.scoretracker-worker.local", short_name.to_lowercase())
    }

    /// Initialize a new worker structure with a default config and a random name.
    pub fn start_default() -> Result<(), WorkerStartError> {
        Self::start(names::random_name().to_owned(), WorkerConfig::load()?)
    }

    /// Initialize a new worker structure, connect to the queue, start ipc threads, and start the worker.
    #[named]
    pub fn start(short_name: String, config: WorkerConfig) -> Result<(), WorkerStartError> {
        log_fn_name!("worker" : auto);

        let pid = process::id();
        let full_name = Worker::make_full_name(&short_name, pid);
        info!("creating worker with name: '{full_name}'");

        let listener = TcpListener::bind("127.0.0.1:0").map_err(WorkerStartError::TcpListenerBindError)?;
        let address = listener.local_addr().map_err(WorkerStartError::TcpListenerLocalAddrError)?;
        let info = WorkerInfo {
            full_name,
            short_name,
            pid,
            birth_timestamp: NsTimestamp::now(),
            address,
        };
        open_log_file(&log_dir().join(info.log_filename())).map_err(WorkerStartError::LogError)?;

        // Connect to queue server/file
        let queue_connection = AssertUnwindSafe(Mutex::new(config.create_queue_connection(&info)?));

        // Create thread communication channels for updating worker status and task progress
        let (worker_status_tx, worker_status_rx) = crossbeam_channel::unbounded();
        let (task_progress_tx, task_progress_rx) = crossbeam_channel::unbounded();
        let worker_status_rx = Arc::new(worker_status_rx);
        let task_progress_rx = Arc::new(task_progress_rx);

        // Create worker structure
        let worker = Worker {
            config,
            data: Arc::new(WorkerData {
                info,
                task_progress: AssertUnwindSafe(Mutex::new(None)),
                status: AssertUnwindSafe(Mutex::new(WorkerStatus::default())),
            }),
            worker_status_tx,
            task_progress_tx,
            queue_connection,
            internal_libraries: LibraryTab::load().expect("todo error handling").scan(),
        };

        // Start a local TCP listener for IPC
        start_listener_thread(
            listener,
            Arc::clone(worker.data()),
            Arc::clone(&worker_status_rx),
            Arc::clone(&task_progress_rx),
        );

        // Connect to the central server
        start_server_connection_thread(Arc::clone(&worker_status_rx), Arc::clone(&task_progress_rx));

        // Start main loop
        worker.main_loop();
        Ok(())
    }

    #[named]
    fn main_loop(self) {
        log_fn_name!(auto);

        // Main part
        match self.config.mode {
            WorkerMode::Single => info!("created single worker, now taking on a task..."),
            WorkerMode::Volatile => info!("created volatile worker, now taking on tasks..."),
            WorkerMode::Persistent => info!("created persistent worker, now taking on tasks..."),
        }

        let worker = Arc::new(self);

        smol::block_on(async move {
            loop {
                if let Some(task_to_do) = worker.take_task().await.expect("todo error handling") {
                    worker.clone().execute_task(task_to_do).await;
                } else {
                    info!("no tasks to do!");
                    if worker.config.mode.quit_when_no_tasks() {
                        break;
                    }
                }

                // Do not jump back up if the mode does not have looping enabled
                if !worker.config.mode.enable_loop() {
                    break;
                }

                // let the worker rest a little bit...
                let seconds = worker.config.sleep_duration_seconds;
                info!("worker sleeping for {seconds} seconds...");
                thread::sleep(Duration::from_secs_f64(seconds));
            }
        });

        info!("exiting");
    }

    /// Execute a task from the queue in the current thread.
    ///
    /// Please note that executing a task may take a long time.
    ///
    /// This function will mark the task as being worked on and write to the [`TaskQueue`] file using [`lockfile`];
    /// only after marking the task in the queue will the task start being executed.
    /// After the task finishes, the results of the task are written automatically to the queue file.
    #[named]
    pub async fn execute_task(self: Arc<Self>, task: Task) {
        log_fn_name!("worker" : auto);
        info!("taking on task with uuid: {}", task.uuid.0);
        info!("starting task: {:?}", task);

        // Update worker state
        {
            let mut status = self.data.status.lock().await;
            status.state = WorkerState::Working;
            status.current_task = Some(task.uuid);
        }

        // Do the task
        let result = self.clone().execute_task_body(&task).await;
        let finish_timestamp = NsTimestamp::now();

        // Send an update about the state of the task to the queue file/server
        match result {
            Ok(success) => {
                self.update_task_state(task.uuid.0, TaskResult::Success(success), finish_timestamp);
            }
            Err(fail) => {
                self.update_task_state(task.uuid.0, TaskResult::Error(fail), finish_timestamp);
            }
        }

        // Update worker state
        {
            let mut status = self.data.status.lock().await;
            status.state = WorkerState::Finished;
            status.current_task = None;
        }
    }

    /// Execute a task in the current thread.
    ///
    /// Please note that executing a task may take a long time.
    ///
    /// The task should be marked as "being worked on" before executing this method to prevent other processes from doing the same task.
    /// The queue file should be written to before calling this method.
    ///
    /// The result of the task should also be saved to the queue after this method finishes, so that no data is lost and the task is not done twice.
    #[named]
    async fn execute_task_body(self: Arc<Self>, task: &Task) -> Result<Success, Fail> {
        log_fn_name!("worker" : auto);

        let result = panic::catch_unwind(move || smol::block_on(task.job.run(self)));
        match result {
            Ok(no_panic) => match no_panic {
                Ok(success) => {
                    success!("task finished successfully: uuid: {}, results: {:#?}", task.uuid.0, success);
                    return Ok(success);
                }
                Err(fail) => {
                    error!("task failed: uuid: {}, reason: {} ({:?})", task.uuid.0, fail, fail);
                    return Err(fail);
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
                return Err(job::Fail::Panic(disp_panic));
                // panic::resume_unwind(panic)
            }
        }
    }

    /// Fetch a task from the queue and mark it as being worked on.
    ///
    /// This function will either connect to the server or use the local queue file. In both cases, the task will be marked as being worked on by this function.
    pub async fn take_task(&self) -> Result<Option<Task>, WorkerError> {
        self.queue_connection.lock().await.take_task(Cow::Borrowed(self.info())).await
    }

    /// Fetch a task with a given UUID and mark it as being worked on.
    ///
    /// This function will either connect to the server or use the local queue file. In both cases, the task will be marked as being worked on by this function.
    pub async fn take_task_with_uuid(&self, task_uuid: Uuid) -> Result<Option<Task>, WorkerError> {
        self.queue_connection
            .lock()
            .await
            .take_task_with_uuid(task_uuid, Cow::Borrowed(self.info()))
            .await
    }

    pub async fn update_task_state(&self, task_uuid: Uuid, result: TaskResult, finish_timestamp: NsTimestamp) -> Result<(), WorkerError> {
        self.queue_connection
            .lock()
            .await
            .update_task_state(task_uuid, result, finish_timestamp, Cow::Borrowed(self.info()))
            .await
    }

    /// Fetches information about a library entry from the server.
    pub async fn fetch_library_entry_by_uuid(&self, proof_uuid: Uuid) -> Result<Option<LibraryEntry>, WorkerError> {
        todo!()
    }

    /// Returns a file path to the root of the specified library, if it is locally available (defined in the [`LibraryTable`] file).
    pub async fn library_local_dir(&self, domain: &LibraryDomain) -> Option<&LibraryAccessPath> {
        let internal_library_connections = &self.internal_libraries;
        internal_library_connections.get_main_path(&domain)
    }

    /// Fetches a proof from a proof library using the provided URL.
    ///
    /// This function may take a long time to finish as the file may need to be downloaded from an external server.
    /// For local libraries, the file path is passed directly; it is not copied. Please use with care - do not write to the file.
    /// The returned library entry should always contain the provided URL.
    pub async fn find_or_download_proof_file(&self, url: &StplUrl) -> Result<(PathBuf, LibraryEntry), WorkerError> {
        if let Some(library_local_dir) = self.library_local_dir(&url.domain).await {
            let resource_path = url
                .path
                .as_ref()
                .expect("todo error handling: stpl url has to have a resource path");
            let file_path = library_local_dir.path.join(resource_path);
            Ok((file_path, todo!()))
        } else {
            todo!()
        }
    }

    /// Uploads a specified file into the specified library location and scans it in.
    ///
    /// This function may take a long time to finish as the file may need to be uploaded from an external server.
    /// For local libraries, the file is moved directly if possible; it may not be copied. Please use with care.
    /// The returned library entry should always contain the provided URL.
    pub async fn move_or_upload_proof_file(
        &self,
        file_to_upload: &Path,
        target_location: &StplUrl,
        entry_mutator: impl Fn(&mut LibraryEntry),
    ) -> Result<LibraryEntry, WorkerError> {
        todo!()
    }

    /// Creates a new unique file path for temporary files or files generated by the worker.
    pub fn create_temp_path(&self) -> PathBuf {
        todo!()
    }

    /// Creates a new unique file path for temporary files or files generated by the worker.
    /// If the provided URL points to a internal library, the returned path will be on the same filesystem as the library if possible, so that it can be moved effortlessly later.
    pub fn create_temp_path_for(&self, url: &StplUrl) -> PathBuf {
        todo!()
    }
}
