use crate::cmd::{self, CmdError};
use chrono::{DateTime, Local};
use function_name::named;
use scoretracker::hive::job::Job;
use scoretracker::hive::jobs::cut_library_video::CutLibraryVideoJob;
use scoretracker::hive::jobs::process_library_video::{Operation, ProcessLibraryVideoJob};
use scoretracker::hive::task::Task;
use scoretracker::hive::{queue::TaskQueue, worker::Worker};
use scoretracker::info_npr;
use scoretracker::util::filelocked::FileLockableDataDefault;
use scoretracker::util::lossless_cut_project::LlcProj;
use scoretracker::util::timestamp::NsLocalTimestamp;
use scoretracker::{config::Config, error, info, log_fn_name, success};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

#[named]
pub fn spawn_worker(persistent: bool) -> Result<(), CmdError> {
    log_fn_name!("cmd" : auto);

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
                    break; // TODO: detect  if there are no tasks, and wait for new tasks in that case (for persistent workers only)
                }
            }

            if !persistent {
                break;
            }

            // let the worker rest a little bit...
            info!("worker sleeping for 5 seconds...");
            thread::sleep(Duration::from_secs(5));
        }
    });

    info!("exiting");
    Ok(())
}

#[named]
pub fn add_task(job: impl Job) -> Result<(), CmdError> {
    log_fn_name!("cmd" : auto);

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

#[named]
pub fn add_task_execute_llc(source_path: PathBuf) -> Result<(), CmdError> {
    log_fn_name!(auto);

    let file_stem = source_path.file_stem().expect("todo: invalid file name").to_string_lossy();

    let llc_proj_filename = format!("{file_stem}-proj.llc");
    let llc_proj_path = source_path.with_file_name(llc_proj_filename);
    info!("loading llc project from: {llc_proj_path:?}");
    let llc = LlcProj::load_from_file(llc_proj_path).expect("todo: invalid llc proj");

    for (i, segment) in llc.cut_segments.iter().enumerate() {
        let segment_number = if llc.cut_segments.len() == 1 { None } else { Some(i + 1) };
        let segment_fragment = segment_number.map(|num| format!("-seg{num}")).unwrap_or_default();
        let cut_start_point = NsLocalTimestamp::from_secs_f64(segment.start);
        let cut_end_point = NsLocalTimestamp::from_secs_f64(segment.end);

        //{file_stem}-00.07.40.660-00.09.48.941-stcut.mkv
        //{file_stem}-00.07.40.660-00.09.48.941-seg4-stcut.mkv
        let file_name = format!(
            "{file_stem}-{}-{}{segment_fragment}-stcut.mkv",
            cut_start_point.to_string_within_filename(),
            cut_end_point.to_string_within_filename()
        );
        let destination_path: PathBuf = source_path.with_file_name(file_name);

        cmd::hive::add_task(CutLibraryVideoJob {
            source_path: source_path.to_path_buf(),
            source_proof_uuid_precondition_check: None,
            cut_start_point: Some(cut_start_point),
            cut_end_point: Some(cut_end_point),
            destination_path,
        })?;
    }
    Ok(())
}

#[named]
pub fn add_task_fold_video(source_path: PathBuf) -> Result<(), CmdError> {
    let file_name = format!(
        "{}-stfolded.mkv",
        source_path.file_stem().expect("todo: invalid file name").to_string_lossy()
    );
    let destination_path: PathBuf = source_path.with_file_name(file_name);
    cmd::hive::add_task(ProcessLibraryVideoJob {
        source_path,
        source_proof_uuid_precondition_check: None,
        operation: Operation::CompressFoldVideo,
        destination_path,
    })
}

#[named]
pub fn add_task_mess_up_video(source_path: PathBuf) -> Result<(), CmdError> {
    let file_name = format!(
        "{}-stmessy.mkv",
        source_path.file_stem().expect("todo: invalid file name").to_string_lossy()
    );
    let destination_path: PathBuf = source_path.with_file_name(file_name);
    cmd::hive::add_task(ProcessLibraryVideoJob {
        source_path,
        source_proof_uuid_precondition_check: None,
        operation: Operation::CompressMessUpVideo,
        destination_path,
    })
}

#[named]
pub fn add_task_crumple_video(source_path: PathBuf) -> Result<(), CmdError> {
    let file_name = format!(
        "{}-stcrumpled.mkv",
        source_path.file_stem().expect("todo: invalid file name").to_string_lossy()
    );
    let destination_path: PathBuf = source_path.with_file_name(file_name);
    cmd::hive::add_task(ProcessLibraryVideoJob {
        source_path,
        source_proof_uuid_precondition_check: None,
        operation: Operation::CompressCrumpleVideo,
        destination_path,
    })
}

#[named]
pub fn add_task_shred_video(source_path: PathBuf) -> Result<(), CmdError> {
    let file_name = format!(
        "{}-stshredded.mkv",
        source_path.file_stem().expect("todo: invalid file name").to_string_lossy()
    );
    let destination_path: PathBuf = source_path.with_file_name(file_name);
    cmd::hive::add_task(ProcessLibraryVideoJob {
        source_path,
        source_proof_uuid_precondition_check: None,
        operation: Operation::CompressShredVideo,
        destination_path,
    })
}
