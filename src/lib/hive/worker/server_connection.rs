use std::sync::Arc;

use crate::{
    data::library::{database::LibraryEntry, stpl_url::StplUrl},
    hive::task::{Task, TaskResult},
    util::timestamp::NsTimestamp,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ServerError {}

// TODO
#[derive(Debug)]
pub enum ServerConnection {}

impl ServerConnection {
    pub async fn connect() -> Result<Self, ServerError> {
        todo!()
    }

    pub async fn get_library_entry_by_url(self: &Arc<Self>, url: &StplUrl) -> Result<LibraryEntry, ServerError> {
        todo!()
    }

    // GET /api/hive/take_task
    // This endpoint should have authentication.
    // `worker_info` should be sent to the server beforehand, during the initial worker authentication, with websockets.
    // Here, we only really need to send the auth token.
    pub async fn take_task(self: &Arc<Self>) -> Result<Option<Task>, ServerError> {
        todo!()
    }

    // GET /api/hive/take_task/{uuid}
    // This endpoint should have authentication.
    // `worker_info` should be sent to the server beforehand, during the initial worker authentication, with websockets.
    // Here, we only really need to send the auth token.
    pub async fn take_task_with_uuid(self: &Arc<Self>, uuid: Uuid) -> Result<Option<Task>, ServerError> {
        todo!()
    }

    // PUT /api/hive/finish_task/{uuid}
    // This endpoint should have authentication.
    // `worker_info` should be sent to the server beforehand, during the initial worker authentication, with websockets.
    // Here, we only really need to send the auth token.
    pub async fn update_task_state(
        self: &Arc<Self>,
        uuid: Uuid,
        result: TaskResult,
        finish_timestamp: NsTimestamp,
    ) -> Result<(), ServerError> {
        todo!()
    }
}
