use crate::hive::queue::TaskQueue;
use crate::hive::task::{Task, TaskResult, TaskState};
use crate::hive::worker::WorkerError;
use crate::hive::worker::data::WorkerInfo;
use crate::util::filelocked::ClosedOrOpen;
use crate::util::timestamp::NsTimestamp;
use std::borrow::Cow;
use uuid::Uuid;

// TODO
#[derive(Debug)]
pub enum ServerConnection {}

impl ServerConnection {
    // GET /api/hive/take_task
    // This endpoint should have authentication.
    // `worker_info` should be sent to the server beforehand, during the initial worker authentication, with websockets.
    // Here, we only really need to send the auth token.
    pub async fn take_task(&self) -> Result<Option<Task>, WorkerError> {
        todo!()
    }

    // GET /api/hive/take_task/{uuid}
    // This endpoint should have authentication.
    // `worker_info` should be sent to the server beforehand, during the initial worker authentication, with websockets.
    // Here, we only really need to send the auth token.
    pub async fn take_task_with_uuid(&self, uuid: Uuid) -> Result<Option<Task>, WorkerError> {
        todo!()
    }

    // PUT /api/hive/finish_task/{uuid}
    // This endpoint should have authentication.
    // `worker_info` should be sent to the server beforehand, during the initial worker authentication, with websockets.
    // Here, we only really need to send the auth token.
    pub async fn update_task_state(&self, uuid: Uuid, result: TaskResult, finish_timestamp: NsTimestamp) -> Result<(), WorkerError> {
        todo!()
    }
}

#[derive(Debug)]
pub enum QueueConnection {
    QueueFile(ClosedOrOpen<TaskQueue>),
    ServerConnection(ServerConnection),
}

impl QueueConnection {
    /// Fetch a task from the queue and mark it as being worked on.
    ///
    /// This function will either connect to the server or use the local queue file. In both cases, the task will be marked as being worked on by this function.
    pub async fn take_task(&mut self, worker_info: Cow<'_, WorkerInfo>) -> Result<Option<Task>, WorkerError> {
        match self {
            Self::QueueFile(file) => {
                let task_queue = file.open().map_err(WorkerError::CannotReopenQueueFile)?;
                let Some(task) = task_queue.top_queued_task_mut() else {
                    return Ok(None);
                };

                task.state = TaskState::Working;
                task.start_timestamp = Some(NsTimestamp::now());
                task.worker_info = Some(worker_info.into_owned());

                let ret = task.clone();
                file.save_and_close().map_err(WorkerError::CannotWriteQueueFile)?;

                Ok(Some(ret))
            }
            Self::ServerConnection(conn) => {
                // If task is Some, the server modifies the task state to TaskState::Working, we don't have to do much
                conn.take_task().await
            }
        }
    }

    /// Fetch a task with a given UUID and mark it as being worked on.
    ///
    /// This function will either connect to the server or use the local queue file. In both cases, the task will be marked as being worked on by this function.
    pub async fn take_task_with_uuid(&mut self, task_uuid: Uuid, worker_info: Cow<'_, WorkerInfo>) -> Result<Option<Task>, WorkerError> {
        match self {
            Self::QueueFile(file) => {
                let task_queue = file.open().map_err(WorkerError::CannotReopenQueueFile)?;
                let Some(task) = task_queue.get_task_mut(task_uuid) else {
                    return Ok(None);
                };

                task.state = TaskState::Working;
                task.start_timestamp = Some(NsTimestamp::now());
                task.worker_info = Some(worker_info.into_owned());

                let ret = task.clone();
                file.save_and_close().map_err(WorkerError::CannotWriteQueueFile)?;

                Ok(Some(ret))
            }
            Self::ServerConnection(conn) => {
                // If task is Some, the server modifies the task state to TaskState::Working, we don't have to do much
                conn.take_task_with_uuid(task_uuid).await
            }
        }
    }

    pub async fn update_task_state(
        &mut self,
        task_uuid: Uuid,
        result: TaskResult,
        finish_timestamp: NsTimestamp,
        worker_info: Cow<'_, WorkerInfo>,
    ) -> Result<(), WorkerError> {
        match self {
            Self::QueueFile(file) => {
                let task_queue = file.open().map_err(WorkerError::CannotReopenQueueFile)?;
                let task = task_queue.get_task_mut(task_uuid).expect("todo error handling");

                task.state = result.state();
                task.result = Some(result);
                task.finish_timestamp = Some(finish_timestamp);

                file.save_and_close().map_err(WorkerError::CannotWriteQueueFile)?;
                Ok(())
            }
            QueueConnection::ServerConnection(conn) => {
                // The server should take care of everything from here really
                conn.update_task_state(task_uuid, result, finish_timestamp).await
            }
        }
    }
}
