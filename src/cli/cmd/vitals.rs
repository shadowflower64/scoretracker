use crate::cmd::CmdError;
use crate::cmd::vitals::LibraryEntryCheckError::{ClipNotFound, ClothNotFound, DryNotFound, InvalidSHA256Hash};
use regex::Regex;
use scoretracker::config::Config;
use scoretracker::data::library::database::{LibraryDatabase, LibraryEntry};
use scoretracker::hive::queue::TaskQueue;
use scoretracker::hive::task::TaskState;
use scoretracker::util::filelocked::{FileLockableData, FileLockableDataWithDefaultPath};
use scoretracker::util::lockfile;
use scoretracker::util::terminal_colors::{ANSI_COLOR_BOLD_GREEN, ANSI_COLOR_BOLD_RED, ANSI_COLOR_RESET, ANSI_ERASE_TO_END};
use scoretracker::util::terminal_colors::{ANSI_COLOR_BOLD_YELLOW, ansi_move_cursor_left};
use scoretracker::util::uuid::UuidString;
use std::fmt;
use std::io::{Write, stdout};
use std::path::PathBuf;
use std::sync::LazyLock;
use thiserror::Error;

fn print_check_name(msg: &str) {
    print!("{msg:.<90}");
}

fn print_check_status(status: &str) {
    let movement = ansi_move_cursor_left(status.len() as u32);
    print!("{ANSI_ERASE_TO_END}{status}{movement}");
    stdout().flush().expect("could not flush stdout");
}

fn print_check_ok() {
    println!("{ANSI_ERASE_TO_END}{ANSI_COLOR_BOLD_GREEN}ok{ANSI_COLOR_RESET}");
}

fn print_check_ok_msg(msg: &str) {
    println!("{ANSI_ERASE_TO_END}{ANSI_COLOR_BOLD_GREEN}ok: {ANSI_COLOR_RESET}{msg}");
}

fn print_check_warn<E: fmt::Display>(warning: &E) {
    println!("{ANSI_ERASE_TO_END}{ANSI_COLOR_BOLD_YELLOW}warning: {ANSI_COLOR_RESET}{warning}");
}

fn print_check_warn_msg(msg: &str) {
    println!("{ANSI_ERASE_TO_END}{ANSI_COLOR_BOLD_YELLOW}warning: {ANSI_COLOR_RESET}{msg}");
}

fn print_check_err<E: fmt::Display>(error: &E) {
    println!("{ANSI_ERASE_TO_END}{ANSI_COLOR_BOLD_RED}error: {ANSI_COLOR_RESET}{error}");
}

#[derive(Error, Debug)]
pub enum LibraryDatabaseCheckError {
    #[error("lockfile error: {0}")]
    Lockfile(#[from] lockfile::Error),
    #[error("entry with uuid {uuid}: {e}")]
    Entry { uuid: UuidString, e: LibraryEntryCheckError },
}

#[derive(Error, Debug)]
pub enum LibraryEntryCheckError {
    #[error("invalid sha256 hash: {0}")]
    InvalidSHA256Hash(String),
    #[error("cloth entry not found: {0}")]
    ClothNotFound(UuidString),
    #[error("dry entry not found: {0}")]
    DryNotFound(UuidString),
    #[error("clip entry not found: {0}")]
    ClipNotFound(UuidString),
}

fn check_entry(library_entry: &LibraryEntry, library_db: &LibraryDatabase) -> Result<(), LibraryEntryCheckError> {
    static SHA256_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9a-f]{64}$").expect("could not compile regex"));
    if !SHA256_REGEX.is_match(&library_entry.sha256) {
        return Err(InvalidSHA256Hash(library_entry.sha256.clone()));
    }
    if let Some(cloth) = &library_entry.cloth {
        library_db.find_entry_by_uuid(cloth.uuid).ok_or(ClothNotFound(cloth.uuid))?;
    }
    if let Some(dry_uuid) = library_entry.dry {
        library_db.find_entry_by_uuid(dry_uuid).ok_or(DryNotFound(dry_uuid))?;
    }
    if let Some(clips) = &library_entry.clips {
        for clip_uuid in clips {
            library_db.find_entry_by_uuid(*clip_uuid).ok_or(ClipNotFound(*clip_uuid))?;
        }
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum TaskQueueCheckError {
    #[error("lockfile error: {0}")]
    Lockfile(#[from] lockfile::Error),
    #[error("task queue has a lot of finished entries ({finished}/{threshold}), consider archiving the task queue to increase performance")]
    Overfill { finished: usize, threshold: usize },
}

fn check_library_database(library_db_path: PathBuf) -> Result<(), LibraryDatabaseCheckError> {
    print_check_name(&format!("checking library database"));
    print_check_status("waiting for filelock...");
    let library_db = LibraryDatabase::lock_and_read(library_db_path, None)?;
    let entry_count = library_db.entries.len();
    print_check_ok_msg(&format!("{entry_count} library entries"));

    print_check_name("checking library database entries");
    for (i, entry) in library_db.entries.iter().enumerate() {
        print_check_status(&format!("({}/{entry_count})", i + 1));
        check_entry(entry, &library_db).map_err(|e| LibraryDatabaseCheckError::Entry { uuid: entry.uuid, e })?;
    }
    Ok(())
}

fn check_task_queue(task_queue_path: PathBuf) -> Result<(), LibraryDatabaseCheckError> {
    const THRESHOLD: usize = 500;

    print_check_name(&format!("checking task queue"));
    print_check_status("waiting for filelock...");
    let task_queue = TaskQueue::lock_and_read(task_queue_path, None)?;
    let total = task_queue.total_count();
    let finished = {
        let queued = task_queue.count(TaskState::Queued);
        let ongoing = task_queue.count(TaskState::Working);
        let paused = task_queue.count(TaskState::Paused);
        let failed = task_queue.count(TaskState::Failed);
        let done = task_queue.count(TaskState::Done);
        print_check_ok_msg(&format!(
            "{total} task entries: {queued} queued, {ongoing} ongoing, {paused} paused, {failed} failed, {done} done"
        ));
        failed + done
    };

    print_check_name(&format!("checking task queue size"));
    if finished > THRESHOLD {
        print_check_warn(&TaskQueueCheckError::Overfill {
            finished,
            threshold: THRESHOLD,
        });
    } else {
        print_check_ok_msg(&format!("finished entries: {finished}/{THRESHOLD}"));
    }

    print_check_name("checking task queue entries");
    for (i, _entry) in task_queue.tasks.iter().enumerate() {
        print_check_status(&format!("({}/{total})", i + 1));
        // check_task(entry, &library_db).map_err(|e| LibraryDatabaseCheckError::Entry { uuid: entry.uuid, e })?;
    }

    Ok(())
}

pub fn check_all() -> Result<(), CmdError> {
    let result_wrapper = |result: Result<_, _>| {
        match result {
            Ok(_) => {
                print_check_ok();
            }
            Err(e) => {
                print_check_err(&e);
            }
        };
    };

    let config_path = Config::default_path();
    println!("config located at: {config_path:?}");
    print_check_name(&format!("checking config"));

    let config = match Config::load() {
        Ok(config) => {
            print_check_ok();
            config
        }
        Err(e) => {
            print_check_err(&e);
            return Err(CmdError::ConfigReadError(e));
        }
    };

    let library_db_path = config.library_database_path();
    println!("library database located at: {library_db_path:?}");
    result_wrapper(check_library_database(library_db_path));

    let task_queue_path = config.task_queue_path();
    println!("task queue located at: {task_queue_path:?}");
    result_wrapper(check_task_queue(task_queue_path));

    // TODO: add matches table, performances table; cross check performance-match foreign keys, player uuids, proof uuids, and song ids

    Ok(())
}
