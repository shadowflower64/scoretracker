use crate::cmd;
use scoretracker::{error, hive::worker::Worker, info, log_fn_name, success};

pub fn spawn_worker(persistent: bool) -> Result<(), cmd::Error> {
    log_fn_name!("cmd:spawn_worker");

    let worker = Worker::new_default()?;
    if persistent {
        info!("created persistent worker, now taking on tasks...");
    } else {
        info!("created volatile worker, now taking on a task...");
    }

    loop {
        match worker.take_on_task() {
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

    info!("exiting");
    Ok(())
}
