use crate::cmd::CmdError;
use chrono::{DateTime, Local};
use scoretracker::hive::job::Job;
use scoretracker::hive::task::Task;
use scoretracker::hive::{queue::TaskQueue, worker::Worker};
use scoretracker::info_npr;
use scoretracker::util::filelocked::FileLockableDataDefault;
use scoretracker::{config::Config, error, info, log_fn_name, success};
use std::sync::Arc;
use std::time::SystemTime;

pub fn spawn_worker(persistent: bool) -> Result<(), CmdError> {
    log_fn_name!("cmd:spawn_worker");

    let worker = Arc::new(Worker::new_default()?);
    if persistent {
        info!("created persistent worker, now taking on tasks...");
    } else {
        info!("created volatile worker, now taking on a task...");
    }

    smol::block_on(async {
        loop {
            match worker.clone().take_on_task().await {
                Ok(_) => {
                    success!("worker task finished successfully");
                }
                Err(e) => {
                    error!("worker task returned error: {e}");
                    break;
                }
            }

            if !persistent {
                break;
            }
        }
    });

    info!("exiting");
    Ok(())
}

pub fn add_task(job: impl Job) -> Result<(), CmdError> {
    log_fn_name!("cmd:add_task");

    let job = job.into_any();
    let config = Config::load().map_err(CmdError::ConfigReadError)?;
    let mut task_queue = TaskQueue::lock_and_read_or_default(config.task_queue_path(), None).map_err(CmdError::TaskQueueOpenError)?;
    let time_identifier = DateTime::<Local>::from(SystemTime::now()).format("%Y%m%d%H%M%S%3f");
    let task = Task::new(format!("Manually added task #{time_identifier}"), job);
    info!("adding task: {task:?}");
    task_queue
        .add_task(task)
        .expect("task was newly created, the uuid should never collide");
    task_queue.save_and_unlock().map_err(CmdError::TaskQueueWriteError)?;
    info_npr!("successfully added task to queue");
    Ok(())
}
