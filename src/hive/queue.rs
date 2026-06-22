use crate::hive::task::{Task, TaskState};
use crate::util::file_ex::{self, FileEx};
use crate::util::filelocked::FileLockableData;
use thiserror::Error;
use uuid::Uuid;

/// A queue of tasks that are to be executed by workers.
///
/// Tasks in this queue are taken on and executed by workers ([`crate::hive::worker::Worker`]).
/// You can implement your own worker processes (in any programming language),
/// but to make sure that no data is lost and no duplicate work is done, please follow the process described below:
///
/// If you want to add a task to the queue:
/// 1. Lock the queue file (the process of locking a file is described in the [`crate::util::lockfile`] module).
/// 2. Read the queue file (jsonlines format) (this is technically optional, as with jsonlines you can add new entries without parsing the whole file).
/// 3. Append a new entry with a unique UUID to the end of the file.
/// 4. Write to the queue file.
/// 5. Unlock the queue file.
///
/// If you want to execute a task in the queue:
/// 1. Lock the queue file.
/// 2. Read the queue file (jsonlines format).
/// 3. Find a task to do, either by an externally provided UUID, or by just choosing one of the tasks.
/// 4. Make sure the state of the task is set to [`TaskState::Queued`].
///    If it is not, then that means that the task is being executed by another process, or it is already done.
/// 5. Set the state of the task to [`TaskState::Working`]
/// 6. Set the [`Task::start_timestamp`] and [`Task::worker_info`] fields of the task to correct values.
/// 7. Save the updated task info to the queue file.
/// 8. Unlock the queue file.
/// 9. Execute the job described in the task.
///    After the task is finished (with either a failure or a success state), continue with the steps below.
/// 10. Lock the queue file.
/// 11. Read the queue file again (it may have changed by now!).
/// 12. Find the task with the same UUID as before.
/// 13. Update the task's state to [`TaskState::Done`] or [`TaskState::Failed`].
/// 14. If the task was successful, update the [`Task::result`] field to the result of the task.
/// 15. Update the [`Task::finish_timestamp`] field to the current timestamp.
/// 16. Save the updated task info to the queue file.
/// 17. Unlock the queue file.
#[derive(Debug, Default)]
pub struct TaskQueue {
    tasks: Vec<Task>,
}

#[derive(Debug, Error)]
#[error("task with the same UUID already exists: {0}")]
pub struct TaskAlreadyExists(Uuid);

#[derive(Debug, Error)]
#[error("task with this UUID was not found: {0}")]
pub struct TaskNotFound(Uuid);

impl TaskQueue {
    pub const STANDARD_FILENAME: &str = "task_queue.jsonl";

    pub fn top_queued_task(&self) -> Option<&Task> {
        self.tasks.iter().find(|task| task.state == TaskState::Queued)
    }

    pub fn top_queued_task_mut(&mut self) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|task| task.state == TaskState::Queued)
    }

    /// Add a new task.
    ///
    /// This function adds a new task to the end of the queue.
    ///
    /// # Errors
    /// If the queue already has a task with the same UUID, nothing happens and an Err variant is returned.
    pub fn add_task(&mut self, task: Task) -> Result<(), TaskAlreadyExists> {
        if self.get_task(task.uuid.0).is_none() {
            self.tasks.push(task);
            Ok(())
        } else {
            Err(TaskAlreadyExists(task.uuid.0))
        }
    }

    /// Update an existing task.
    ///
    /// This function updates an existing task in the queue by finding the task with the same UUID and replacing it with the given task data.
    ///
    /// # Errors
    /// If the queue does not already have a task with the same UUID, nothing happens and an Err variant is returned.
    pub fn update_task(&mut self, task: Task) -> Result<(), TaskNotFound> {
        if let Some(old_task) = self.get_task_mut(task.uuid.0) {
            *old_task = task;
            Ok(())
        } else {
            Err(TaskNotFound(task.uuid.0))
        }
    }

    /// Add a new task or update an existing one.
    ///
    /// This function updates an existing task in the queue if a task with the same UUID exists already,
    /// and adds a new task if a task with this UUID is not present in the queue yet.
    ///
    /// This is like a combination of [`Self::add_task`] and [`Self::update_task`].
    pub fn add_or_update_task(&mut self, task: Task) {
        if let Some(old_task) = self.get_task_mut(task.uuid.0) {
            *old_task = task;
        } else {
            self.tasks.push(task);
        }
    }

    pub fn get_task(&self, task_uuid: Uuid) -> Option<&Task> {
        self.tasks.iter().find(|task| task.uuid.0 == task_uuid)
    }

    pub fn get_task_mut(&mut self, task_uuid: Uuid) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|task| task.uuid.0 == task_uuid)
    }
}

impl FileLockableData for TaskQueue {
    fn _inner_read<F: FileEx + ?Sized>(file_ex: &F) -> file_ex::Result<Option<Self>> {
        file_ex.read_from_jsonlines().map(|x| x.map(|y| Self { tasks: y }))
    }
    fn _inner_write<F: FileEx + ?Sized>(&self, file_ex: &F) -> file_ex::Result<()> {
        file_ex.write_as_jsonlines(&self.tasks)
    }
}
