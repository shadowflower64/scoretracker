use crate::cmd::CmdError;
use crate::cmd::vitals::LogCheckError::GetSizeError;
use fs_extra::dir::get_size;
use regex::Regex;
use scoretracker::config::Config;
use scoretracker::data::game::game_instance_from_id;
use scoretracker::data::library::database::{LibraryDatabase, LibraryEntry};
use scoretracker::data::scoreboard::r#match::{MatchDatabase, MatchTrait};
use scoretracker::data::scoreboard::performance::{PerformanceDatabase, PerformanceTrait};
use scoretracker::data::scoreboard::player::{Player, PlayerDatabase};
use scoretracker::hive::queue::TaskQueue;
use scoretracker::hive::task::TaskState;
use scoretracker::util::byte_count::ByteCount;
use scoretracker::util::dirs::log_dir;
use scoretracker::util::filelocked::{FileLockableData, FileLockableDataWithDefaultPath};
use scoretracker::util::lockfile;
use scoretracker::util::terminal_colors::{ANSI_COLOR_BOLD_GREEN, ANSI_COLOR_BOLD_RED, ANSI_COLOR_RESET, ANSI_ERASE_TO_END};
use scoretracker::util::terminal_colors::{ANSI_COLOR_BOLD_YELLOW, ansi_move_cursor_left};
use scoretracker::util::timestamp::NsDuration;
use scoretracker::util::uuid::UuidString;
use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::io::{Write, stdout};
use std::path::Path;
use std::sync::LazyLock;
use std::sync::atomic::Ordering;
use thiserror::Error;

const QUEUE_SIZE_THRESHOLD: usize = 500;
const LOG_BYTES_THRESHOLD: ByteCount = ByteCount::mebibytes(10.0);

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

fn print_check_warn(warning: impl Display) {
    println!("{ANSI_ERASE_TO_END}{ANSI_COLOR_BOLD_YELLOW}warning: {ANSI_COLOR_RESET}{warning}");
}

fn _print_check_warn_msg(msg: &str) {
    println!("{ANSI_ERASE_TO_END}{ANSI_COLOR_BOLD_YELLOW}warning: {ANSI_COLOR_RESET}{msg}");
}

fn print_check_err(error: impl Display) {
    println!("{ANSI_ERASE_TO_END}{ANSI_COLOR_BOLD_RED}error: {ANSI_COLOR_RESET}{error}");
}

#[derive(Debug)]
pub enum ErrorWithImportance<E> {
    Critical(E),
    Warning(E),
}

impl<E> ErrorWithImportance<E> {
    pub fn inner(self) -> E {
        match self {
            Self::Critical(e) => e,
            Self::Warning(e) => e,
        }
    }
}

pub type CheckResult<E> = Result<Option<String>, ErrorWithImportance<E>>;
pub type CheckPartialResult<T, E> = Result<T, ErrorWithImportance<E>>;

impl<E> From<E> for ErrorWithImportance<E> {
    fn from(value: E) -> Self {
        Self::Critical(value)
    }
}

trait CanBecomeAWarning<T, E> {
    #[allow(unused)]
    fn critical(self) -> CheckPartialResult<T, E>;
    fn warn(self) -> CheckPartialResult<T, E>;
}

impl<T, E: Display> CanBecomeAWarning<T, E> for Result<T, E> {
    fn critical(self) -> CheckPartialResult<T, E> {
        match self {
            Ok(a) => Ok(a),
            Err(e) => Err(ErrorWithImportance::Critical(e)),
        }
    }
    fn warn(self) -> CheckPartialResult<T, E> {
        match self {
            Ok(a) => Ok(a),
            Err(e) => Err(ErrorWithImportance::Warning(e)),
        }
    }
}

impl<E> CanBecomeAWarning<Option<String>, E> for CheckResult<E> {
    fn critical(self) -> CheckResult<E> {
        match self {
            Ok(a) => Ok(a),
            Err(x) => Err(ErrorWithImportance::Critical(x.inner())),
        }
    }
    fn warn(self) -> CheckResult<E> {
        match self {
            Ok(a) => Ok(a),
            Err(x) => Err(ErrorWithImportance::Warning(x.inner())),
        }
    }
}

fn result_wrapper<E: Display>(result: CheckResult<E>) {
    match result {
        CheckResult::Ok(Some(a)) => {
            print_check_ok_msg(&a);
        }
        CheckResult::Ok(None) => {
            print_check_ok();
        }
        CheckResult::Err(ErrorWithImportance::Critical(e)) => {
            print_check_err(&e);
        }
        CheckResult::Err(ErrorWithImportance::Warning(e)) => {
            print_check_warn(&e);
        }
    }
}

#[derive(Error, Debug)]
pub enum TaskQueueCheckError {
    #[error("lockfile error: {0}")]
    Lockfile(#[from] lockfile::Error),
    #[error("task queue has a lot of finished entries ({finished}/{threshold}), consider archiving the task queue to increase performance")]
    Overfill { finished: usize, threshold: usize },
}

fn check_task_queue(task_queue_path: &Path) -> CheckResult<TaskQueueCheckError> {
    print_check_name("checking task queue");
    print_check_status("waiting for filelock...");
    use TaskQueueCheckError::Lockfile;
    let task_queue = TaskQueue::lock_and_read(task_queue_path, None).map_err(Lockfile)?;
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

    print_check_name("checking task queue size");
    if finished > QUEUE_SIZE_THRESHOLD {
        print_check_warn(&TaskQueueCheckError::Overfill {
            finished,
            threshold: QUEUE_SIZE_THRESHOLD,
        });
    } else {
        print_check_ok_msg(&format!("finished entries: {finished}/{QUEUE_SIZE_THRESHOLD}"));
    }

    print_check_name("checking task queue entries");
    for (i, _entry) in task_queue.tasks.iter().enumerate() {
        print_check_status(&format!("({}/{total})", i + 1));
        // check_task(entry, &library_db).map_err(|e| LibraryDatabaseCheckError::Entry { uuid: entry.uuid, e })?; // TODO: check stuck tasks
    }

    Ok(None)
}

#[derive(Error, Debug)]
pub enum LogCheckError {
    #[error("get size: {0}")]
    GetSizeError(fs_extra::error::Error),
    #[error("log directory contains a lot of data ({total_size}/{threshold}), consider archiving some logs to increase performance")]
    Overfill { total_size: ByteCount, threshold: ByteCount },
}

fn check_log_dir(log_dir: &Path) -> CheckResult<LogCheckError> {
    print_check_name("checking logs");
    print_check_status("reading dir...");
    let total_size: ByteCount = get_size(log_dir).map_err(GetSizeError)?.into();
    let threshold: ByteCount = LOG_BYTES_THRESHOLD;
    if total_size > threshold {
        Err(LogCheckError::Overfill { total_size, threshold }).warn()
    } else {
        Ok(Some(format!("total size: {total_size}/{threshold}")))
    }
}

#[derive(Error, Debug)]
pub enum LibraryEntryCheckError {
    #[error("reused uuid: {0}")]
    ReusedUuid(UuidString),
    #[error("invalid sha256 hash: {0}")]
    InvalidSHA256Hash(String),
    #[error("duplicate sha256 hash: {0}")]
    DuplicateSHA256Hash(String),
    #[error("invalid youtube id: {0}")]
    InvalidYouTubeId(String),
    #[error("duplicate youtube id: {0}")]
    DuplicateYouTubeId(String),
    #[error("no sha256 hash or youtube id present")]
    NoHashOrYouTubeId,
    #[error("cloth entry not found: {0}")]
    ClothNotFound(UuidString),
    #[error("dry entry not found: {0}")]
    DryNotFound(UuidString),
    #[error("clip entry not found: {0}")]
    ClipNotFound(UuidString),
}

fn check_library_entry(
    entry: &LibraryEntry,
    library_db: &LibraryDatabase,
    uuids: &mut HashSet<UuidString>,
    sha256_hashes: &mut HashSet<String>,
    youtube_ids: &mut HashSet<String>,
) -> Result<(), LibraryEntryCheckError> {
    type E = LibraryEntryCheckError;

    if !uuids.insert(entry.uuid) {
        return Err(E::ReusedUuid(entry.uuid));
    }

    static SHA256_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9a-f]{64}$").expect("could not compile regex"));
    static YOUTUBE_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9A-Za-z-_]{11}$").expect("could not compile regex"));
    if let Some(sha256) = &entry.sha256 {
        if !SHA256_REGEX.is_match(sha256) {
            Err(E::InvalidSHA256Hash(sha256.clone()))?;
        }
        if !sha256_hashes.insert(sha256.clone()) {
            return Err(E::DuplicateSHA256Hash(sha256.clone()));
        }
    } else if let Some(youtube_id) = &entry.youtube_id {
        if !YOUTUBE_ID_REGEX.is_match(youtube_id) {
            Err(E::InvalidYouTubeId(youtube_id.clone()))?;
        }
        if !youtube_ids.insert(youtube_id.clone()) {
            return Err(E::DuplicateYouTubeId(youtube_id.clone()));
        }
    } else {
        Err(E::NoHashOrYouTubeId)?;
    }

    if let Some(cloth) = &entry.cloth {
        library_db.find_entry_by_uuid(cloth.uuid.0).ok_or(E::ClothNotFound(cloth.uuid))?;
    }

    if let Some(dry_uuid) = entry.dry {
        library_db.find_entry_by_uuid(dry_uuid.0).ok_or(E::DryNotFound(dry_uuid))?;
    }

    if let Some(clips) = &entry.clips {
        for clip_uuid in clips {
            library_db.find_entry_by_uuid(clip_uuid.0).ok_or(E::ClipNotFound(*clip_uuid))?;
        }
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum PerformanceCheckError {
    #[error("reused uuid: {0}")]
    ReusedUuid(UuidString),
    #[error("found performances close to each other: {0:?}")]
    ClosePerformances(Vec<(UuidString, NsDuration)>),
    #[error("player not found: {0}")]
    PlayerNotFound(UuidString),
    #[error("match not found: {0}")]
    MatchNotFound(UuidString),
    #[error("library entry not found: {0}")]
    EntryNotFound(UuidString),
    #[error("unknown game id: '{0}'")]
    UnknownGame(String),
    #[error("match game id ('{0}') does not match performance game id ('{1}')")]
    MatchGameDoesNotMatch(String, String),
    #[error("{0}")]
    Custom(String),
}

fn check_performance(
    performance: &dyn PerformanceTrait,
    player_db: &PlayerDatabase,
    match_db: &MatchDatabase,
    performance_db: &PerformanceDatabase,
    library_db: &LibraryDatabase,
    uuids: &mut HashSet<UuidString>,
) -> Result<(), PerformanceCheckError> {
    type E = PerformanceCheckError;

    if !uuids.insert(performance.uuid()) {
        return Err(E::ReusedUuid(performance.uuid()));
    }

    let other_close = performance_db
        .find_close_performances_from_diff_match(performance, NsDuration::from_secs_f64(30.0), match_db)
        .map_err(|x| E::MatchNotFound(x.into()))?;
    if !other_close.is_empty() {
        return Err(E::ClosePerformances(
            other_close.iter().map(|(m, how_close)| (m.uuid(), *how_close)).collect(),
        ));
    }

    let player_uuid = performance.player_uuid();
    let _player = player_db.find_player_by_uuid(player_uuid).ok_or(E::PlayerNotFound(player_uuid))?;

    let match_uuid = performance.match_uuid();
    let match_data = match_db.find_match_by_uuid(match_uuid).ok_or(E::MatchNotFound(match_uuid))?;

    for proof_uuid in performance.proof() {
        let _entry = library_db.find_entry_by_uuid(proof_uuid.0).ok_or(E::EntryNotFound(*proof_uuid))?;
    }

    let game_id = performance.game_id();
    let _game = game_instance_from_id(game_id).ok_or(E::UnknownGame(game_id.to_owned()))?;

    let match_game_id = match_data.game_id();
    if match_game_id != game_id {
        return Err(E::MatchGameDoesNotMatch(match_game_id.to_owned(), game_id.to_owned()));
    };

    performance
        .check_vitals(player_db, match_db, performance_db, library_db)
        .map_err(E::Custom)?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum MatchCheckError {
    #[error("reused uuid: {0}")]
    ReusedUuid(UuidString),
    #[error("found matches close to each other: {0:?}")]
    CloseMatches(Vec<(UuidString, NsDuration)>),
    #[error("library entry not found: {0}")]
    EntryNotFound(UuidString),
    #[error("unknown game id: '{0}'")]
    UnknownGame(String),
    #[error("{0}")]
    Custom(String),
}

fn check_match(
    match_data: &dyn MatchTrait,
    player_db: &PlayerDatabase,
    match_db: &MatchDatabase,
    performance_db: &PerformanceDatabase,
    library_db: &LibraryDatabase,
    uuids: &mut HashSet<UuidString>,
) -> Result<(), MatchCheckError> {
    type E = MatchCheckError;

    if !uuids.insert(match_data.uuid()) {
        return Err(E::ReusedUuid(match_data.uuid()));
    }

    let other_close = match_db.find_other_close_matches(match_data, NsDuration::from_secs_f64(30.0));
    if !other_close.is_empty() {
        return Err(E::CloseMatches(
            other_close.iter().map(|(m, how_close)| (m.uuid(), *how_close)).collect(),
        ));
    }

    // TODO: check song id here

    for proof_uuid in match_data.proof() {
        let _entry = library_db.find_entry_by_uuid(proof_uuid.0).ok_or(E::EntryNotFound(*proof_uuid))?;
    }

    let game_id = match_data.game_id();
    let _game = game_instance_from_id(game_id).ok_or(E::UnknownGame(game_id.to_owned()))?;

    match_data
        .check_vitals(player_db, match_db, performance_db, library_db)
        .map_err(E::Custom)?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum PlayerCheckError {
    #[error("reused uuid: {0}")]
    ReusedUuid(UuidString),
}

fn check_player(player: &Player, uuids: &mut HashSet<UuidString>) -> Result<(), PlayerCheckError> {
    type E = PlayerCheckError;

    if !uuids.insert(player.uuid) {
        return Err(E::ReusedUuid(player.uuid));
    }

    Ok(())
}

#[derive(Error, Debug)]
pub enum ScoreboardCheckError {
    #[error("lockfile error: {0}")]
    Lockfile(#[from] lockfile::Error),
    #[error("entry with uuid {uuid}: {e}")]
    LibraryEntry { uuid: UuidString, e: LibraryEntryCheckError },
    #[error("performance with uuid {uuid}: {e}")]
    Performance { uuid: UuidString, e: PerformanceCheckError },
    #[error("match with uuid {uuid}: {e}")]
    Match { uuid: UuidString, e: MatchCheckError },
    #[error("player with uuid {uuid}: {e}")]
    Player { uuid: UuidString, e: PlayerCheckError },
}

fn check_scoreboard_databases(
    player_db_path: &Path,
    match_db_path: &Path,
    performance_db_path: &Path,
    library_db_path: &Path,
) -> CheckResult<ScoreboardCheckError> {
    type E = ScoreboardCheckError;

    print_check_name("loading player database");
    print_check_status("waiting for player database filelock...");
    let player_db = PlayerDatabase::lock_and_read(player_db_path, None).map_err(E::Lockfile)?;
    let player_count = player_db.players.len();
    print_check_ok_msg(&format!("{player_count} players"));

    print_check_name("loading match database");
    print_check_status("waiting for match database filelock...");
    let match_db = MatchDatabase::lock_and_read(match_db_path, None).map_err(E::Lockfile)?;
    let match_count = match_db.matches.len();
    print_check_ok_msg(&format!("{match_count} matches"));

    print_check_name("loading performance database");
    print_check_status("waiting for performance database filelock...");
    let performance_db = PerformanceDatabase::lock_and_read(performance_db_path, None).map_err(E::Lockfile)?;
    let performance_count = performance_db.performances.len();
    print_check_ok_msg(&format!("{performance_count} performances"));

    print_check_name("loading library database");
    print_check_status("waiting for library database filelock...");
    let library_db = LibraryDatabase::lock_and_read(library_db_path, None).map_err(E::Lockfile)?; // TODO: this is loaded earlier already, could be reused
    let entry_count = library_db.entries.len();
    print_check_ok_msg(&format!("{entry_count} entries"));

    let mut all_mainkey_uuids = HashSet::new();
    let mut all_sha256_hashes = HashSet::new();
    let mut all_youtube_ids = HashSet::new();

    print_check_name("checking library database entries");
    for (i, entry) in library_db.entries.iter().enumerate() {
        print_check_status(&format!("({}/{entry_count})", i + 1));
        check_library_entry(
            entry,
            &library_db,
            &mut all_mainkey_uuids,
            &mut all_sha256_hashes,
            &mut all_youtube_ids,
        )
        .map_err(|e| E::LibraryEntry { uuid: entry.uuid, e })?;
    }
    print_check_ok();

    print_check_name("checking performance database entries");
    for (i, performance) in performance_db.performances.iter().enumerate() {
        print_check_status(&format!("({}/{performance_count})", i + 1));
        check_performance(
            performance.as_ref(),
            &player_db,
            &match_db,
            &performance_db,
            &library_db,
            &mut all_mainkey_uuids,
        )
        .map_err(|e| E::Performance {
            uuid: performance.uuid(),
            e,
        })?;
    }
    print_check_ok();

    print_check_name("checking match database entries");
    for (i, match_data) in match_db.matches.iter().enumerate() {
        print_check_status(&format!("({}/{match_count})", i + 1));
        check_match(
            match_data.as_ref(),
            &player_db,
            &match_db,
            &performance_db,
            &library_db,
            &mut all_mainkey_uuids,
        )
        .map_err(|e| E::Match {
            uuid: match_data.uuid(),
            e,
        })?;
    }
    print_check_ok();

    print_check_name("checking player database entries");
    for (i, player) in player_db.players.iter().enumerate() {
        print_check_status(&format!("({}/{player_count})", i + 1));
        check_player(player, &mut all_mainkey_uuids).map_err(|e| E::Player { uuid: player.uuid, e })?;
    }

    // TODO: add:
    // * matches table,
    // * performances table;
    // * cross check performance-match foreign keys,
    // * player uuids,
    // * proof uuids,
    // * and song ids
    Ok(None)
}

pub fn check_all() -> Result<(), CmdError> {
    // Disable lockfile logging
    lockfile::DEBUG_PRINT.store(false, Ordering::Release);

    let config_path = Config::default_path();
    println!("config located at: {config_path:?}");
    print_check_name("checking config");

    let config = match Config::load() {
        Ok(config) => {
            print_check_ok();
            config
        }
        Err(e) => {
            print_check_err(e);
            return Err(CmdError::ConfigReadError(e));
        }
    };

    let task_queue_path = config.task_queue_path();
    println!("task queue located at: {task_queue_path:?}");
    result_wrapper(check_task_queue(&task_queue_path));

    let log_path = log_dir();
    println!("log directory located at: {log_path:?}");
    result_wrapper(check_log_dir(&log_path));

    let player_db_path = config.player_database_path();
    println!("player database located at: {player_db_path:?}");
    let match_db_path = config.match_database_path();
    println!("match database located at: {match_db_path:?}");
    let performance_db_path = config.performance_database_path();
    println!("performance database located at: {performance_db_path:?}");
    let library_db_path = config.library_database_path();
    println!("library database located at: {library_db_path:?}");
    result_wrapper(check_scoreboard_databases(
        &player_db_path, &match_db_path, &performance_db_path, &library_db_path,
    ));

    Ok(())
}
