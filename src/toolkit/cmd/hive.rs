use crate::cmd::{self, CmdError};
use chrono::{DateTime, Local};
use function_name::named;
use scoretracker::data::library::stpl_url::{LibraryRoot, StplUrl};
use scoretracker::hive::job::Job;
use scoretracker::hive::jobs::cut_library_video::CutLibraryVideoJob;
use scoretracker::hive::jobs::process_library_video::{Operation, ProcessLibraryVideoJob};
use scoretracker::hive::task::Task;
use scoretracker::hive::{queue::TaskQueue, worker::Worker};
use scoretracker::info_npr;
use scoretracker::util::filelocked::FileLockableDataDefault;
use scoretracker::util::lossless_cut_project::LlcProj;
use scoretracker::util::timestamp::NsLocalTimestamp;
use scoretracker::{config::Config, info, log_fn_name};
use std::path::PathBuf;
use std::time::SystemTime;

#[named]
pub fn spawn_worker() -> Result<(), CmdError> {
    Worker::start_default()?;
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

    let root = LibraryRoot::of(&source_path).expect("todo: invalid source path");
    let source = root.url_to(&source_path).expect("todo: invalid source path");
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
        let destination = root.url_to(&destination_path).expect("todo: invalid destination path");

        cmd::hive::add_task(CutLibraryVideoJob {
            source: source.clone(),
            source_proof_uuid_precondition_check: None,
            cut_start_point: Some(cut_start_point),
            cut_end_point: Some(cut_end_point),
            destination,
        })?;
    }
    Ok(())
}

#[named]
pub fn add_task_fold_video(source_path: PathBuf) -> Result<(), CmdError> {
    let root = LibraryRoot::of(&source_path).expect("todo: invalid source path");
    let source = root.url_to(&source_path).expect("todo: invalid source path");
    let file_stem = source_path.file_stem().expect("todo: invalid file name").to_string_lossy();
    let destination_path = source_path.with_file_name(format!("{file_stem}-stfolded.mkv"));
    let destination = root.url_to(&destination_path).expect("todo: invalid destination path");

    cmd::hive::add_task(ProcessLibraryVideoJob {
        source,
        source_proof_uuid_precondition_check: None,
        operation: Operation::CompressFoldVideo,
        destination,
    })
}

#[named]
pub fn add_task_mess_up_video(source_path: PathBuf) -> Result<(), CmdError> {
    let root = LibraryRoot::of(&source_path).expect("todo: invalid source path");
    let source = root.url_to(&source_path).expect("todo: invalid source path");
    let file_stem = source_path.file_stem().expect("todo: invalid file name").to_string_lossy();
    let destination_path = source_path.with_file_name(format!("{file_stem}-stmessy.mkv"));
    let destination = root.url_to(&destination_path).expect("todo: invalid destination path");

    cmd::hive::add_task(ProcessLibraryVideoJob {
        source,
        source_proof_uuid_precondition_check: None,
        operation: Operation::CompressMessUpVideo,
        destination,
    })
}

#[named]
pub fn add_task_crumple_video(source_path: PathBuf) -> Result<(), CmdError> {
    let root = LibraryRoot::of(&source_path).expect("todo: invalid source path");
    let source = root.url_to(&source_path).expect("todo: invalid source path");
    let file_stem = source_path.file_stem().expect("todo: invalid file name").to_string_lossy();
    let destination_path = source_path.with_file_name(format!("{file_stem}-stcrumpled.mkv"));
    let destination = root.url_to(&destination_path).expect("todo: invalid destination path");

    cmd::hive::add_task(ProcessLibraryVideoJob {
        source,
        source_proof_uuid_precondition_check: None,
        operation: Operation::CompressCrumpleVideo,
        destination,
    })
}

#[named]
pub fn add_task_shred_video(source_path: PathBuf) -> Result<(), CmdError> {
    let root = LibraryRoot::of(&source_path).expect("todo: invalid source path");
    let source = root.url_to(&source_path).expect("todo: invalid source path");
    let file_stem = source_path.file_stem().expect("todo: invalid file name").to_string_lossy();
    let destination_path = source_path.with_file_name(format!("{file_stem}-stshredded.mkv"));
    let destination = root.url_to(&destination_path).expect("todo: invalid destination path");

    cmd::hive::add_task(ProcessLibraryVideoJob {
        source,
        source_proof_uuid_precondition_check: None,
        operation: Operation::CompressShredVideo,
        destination,
    })
}
