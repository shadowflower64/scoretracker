//! Module with simple logging macros.
use crate::hive::worker::data::WorkerInfo;
use crate::util::dirs;
use crate::util::terminal_colors::{ANSI_COLOR_BOLD_BLACK, ANSI_COLOR_BOLD_RED, ANSI_COLOR_RESET};
use chrono::{DateTime, Local, SecondsFormat};
use smol::io;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::SystemTime;
use thiserror::Error;

/// Sets the function name used in log macros to the given value.
#[macro_export]
macro_rules! log_fn_name {
    ($arg:literal) => {
        pub const LOG_FN_NAME: &str = $arg;
    };
}

/// Sets whether the debug messages should be printed or not.
#[macro_export]
macro_rules! log_should_print_debug {
    ($boolvalue:expr) => {
        pub const PRINT_DEBUG_MESSAGES: bool = $boolvalue;
    };
    (dynamic: $atomicbool:ident) => {
        pub const PRINT_DEBUG_MESSAGES: &AtomicBool = &$atomicbool;
    };
}

/// Returns a filename for a log file with the current date and time.
pub fn create_general_log_filename() -> String {
    let now: DateTime<Local> = SystemTime::now().into();
    let datetime = now.format("%Y_%m_%d_%H_%M_%S");
    format!("log_{datetime}.log")
}

/// Returns a filename for a log file with the worker's birth date, name and pid.
pub fn create_worker_log_filename(worker_info: &WorkerInfo) -> String {
    worker_info.log_filename()
}

static LOG_FILE: LazyLock<Mutex<Option<File>>> = LazyLock::new(|| Mutex::new(None));

#[derive(Debug, Error)]
pub enum LogError {
    #[error("log file has no parent directory: {path:?}")]
    NoParentPath { path: PathBuf },
    #[error("could not create parent directories for log file: {path:?}; reason: {e}")]
    CreateDirectoriesError { e: io::Error, path: PathBuf },
    #[error("could not open log file: {path:?}; reason: {e}")]
    OpenFileError { e: io::Error, path: PathBuf },
}

/// Changes the active log file to the specified path.
pub fn open_log_file(path: &Path) -> Result<(), LogError> {
    use crate::info;
    log_fn_name!("open_log_file");
    info!("switching log file to: {path:?}");
    let log_file_parent = path.parent().ok_or_else(|| LogError::NoParentPath { path: path.to_path_buf() })?;
    fs::create_dir_all(log_file_parent).map_err(|e| LogError::CreateDirectoriesError {
        e,
        path: path.to_path_buf(),
    })?;
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)
        .map_err(|e| LogError::OpenFileError {
            e,
            path: path.to_path_buf(),
        })?;
    *LOG_FILE.lock().unwrap() = Some(file);
    eprintln!("{ANSI_COLOR_BOLD_BLACK}log file switched to: {path:?}{ANSI_COLOR_RESET}");
    Ok(())
}

/// Changes the active log file to the default path.
pub fn open_default_log_file() -> Result<(), LogError> {
    open_log_file(&dirs::log_dir().join(create_general_log_filename()))
}

/// Prints out a message on `stderr`, with a function name, a thread name, and a timestamp, with the provided log level and color.
/// Also prints out a non-colored message to the log file in [`LOG_FILE`].
pub fn on_log<F: Fn() -> String, G: Fn() -> String>(fmt_plain: F, fmt_colored: G) {
    eprintln!("{}", fmt_colored());

    let mut log_file = LOG_FILE.lock().unwrap();
    if let Some(file) = &mut *log_file
        && let Err(e) = writeln!(file, "{}", fmt_plain())
    {
        eprintln!("{ANSI_COLOR_BOLD_RED}could not write to log file: {e}{ANSI_COLOR_RESET}");
    }
}

/// Returns the current date and time as a string. Used by logging macros.
pub fn datetime_now() -> String {
    let datetime: DateTime<Local> = SystemTime::now().into();
    datetime.to_rfc3339_opts(SecondsFormat::Millis, false)
}

/// Prints out a message on `stderr` (by calling [`on_log`]), with a function name, a thread name, and a timestamp, with the provided log level and color.
#[macro_export]
macro_rules! log_print {
    ($log_level: literal, $log_level_color: ident, $($arg:tt)*) => {{
        #[allow(unused_imports)]
        use $crate::util::terminal_colors::{ANSI_COLOR_BOLD_MAGENTA, ANSI_COLOR_BOLD_BLUE, ANSI_COLOR_BOLD_YELLOW, ANSI_COLOR_BOLD_RED, ANSI_COLOR_BOLD_GREEN, ANSI_COLOR_RESET};
        use $crate::util::log::datetime_now;
        use $crate::util::log::on_log;
        let context_name = if let Some(thread_name) = std::thread::current().name() {
            format!("{thread_name}/{LOG_FN_NAME}")
        } else {
            format!("{LOG_FN_NAME}")
        };
        let datetime_now = datetime_now();
        let log_level_color = $log_level_color;
        let log_level = $log_level;
        let message = format!($($arg)*);
        let fmt_plain = || format!("{datetime_now} {log_level}: [{context_name}] {message}");
        let fmt_colored = || format!("{datetime_now} {log_level_color}{log_level}:{ANSI_COLOR_RESET} [{context_name}] {message}");
        on_log(fmt_plain, fmt_colored)
    }};
}

#[macro_export]
macro_rules! log_print_npr {
    ($log_level: literal, $log_level_color: ident, $($arg:tt)*) => {{
        #[allow(unused_imports)]
        use $crate::util::terminal_colors::{ANSI_COLOR_BOLD_MAGENTA, ANSI_COLOR_BOLD_BLUE, ANSI_COLOR_BOLD_YELLOW, ANSI_COLOR_BOLD_RED, ANSI_COLOR_BOLD_GREEN, ANSI_COLOR_BOLD_CYAN, ANSI_COLOR_RESET};
        use $crate::util::log::on_log;
        let log_level_color = $log_level_color;
        let log_level = $log_level;
        let message = format!($($arg)*);
        let fmt_plain = || format!("{log_level}: {message}");
        let fmt_colored = || format!("{log_level_color}{log_level}:{ANSI_COLOR_RESET} {message}");
        on_log(fmt_plain, fmt_colored)
    }};
}

pub trait AsBool {
    fn as_bool(&self) -> bool;
}

impl AsBool for bool {
    fn as_bool(&self) -> bool {
        *self
    }
}

impl AsBool for &AtomicBool {
    fn as_bool(&self) -> bool {
        self.load(Ordering::Acquire)
    }
}

/// Prints out a magenta debug message on `stderr`, with a prefix containing the function name, a thread name, and a timestamp.
///
/// This log message is printed only if the `PRINT_DEBUG_MESSAGES` flag (set using the [`log_should_print_debug!`] macro) is set to true.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        use $crate::util::log::AsBool;
        if AsBool::as_bool(&PRINT_DEBUG_MESSAGES) {
            use $crate::log_print;
            log_print!("debug", ANSI_COLOR_BOLD_MAGENTA, $($arg)*);
        }
    }};
}

/// Prints out a blue info message on `stderr`, with a prefix containing the function name, a thread name, and a timestamp.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        use $crate::log_print;
        log_print!("info", ANSI_COLOR_BOLD_BLUE, $($arg)*);
    }};
}

/// Prints out a yellow warning message on `stderr`, with a prefix containing the function name, a thread name, and a timestamp.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        use $crate::log_print;
        log_print!("warn", ANSI_COLOR_BOLD_YELLOW, $($arg)*);
    }};
}

/// Prints out a red error message on `stderr`, with a prefix containing the function name, a thread name, and a timestamp.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use $crate::log_print;
        log_print!("error", ANSI_COLOR_BOLD_RED, $($arg)*);
    }};
}

/// Prints out a green success message on `stderr`, with a prefix containing the function name, a thread name, and a timestamp.
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        use $crate::log_print;
        log_print!("success", ANSI_COLOR_BOLD_GREEN, $($arg)*);
    }};
}

/// Prints out a magenta debug message on `stderr` without a prefix. (`npr` stands for "no prefix".)
///
/// This log message is printed only if the `PRINT_DEBUG_MESSAGES` flag (set using the [`log_should_print_debug!`] macro) is set to true.
#[macro_export]
macro_rules! debug_npr {
    ($($arg:tt)*) => {{
        if PRINT_DEBUG_MESSAGES {
            use $crate::log_print_npr;
            log_print_npr!("debug", ANSI_COLOR_BOLD_MAGENTA, $($arg)*);
        }
    }};
}

/// Prints out a blue info message on `stderr` without a prefix. (`npr` stands for "no prefix".)
#[macro_export]
macro_rules! info_npr {
    ($($arg:tt)*) => {{
        use $crate::log_print_npr;
        log_print_npr!("info", ANSI_COLOR_BOLD_BLUE, $($arg)*);
    }};
}

/// Prints out a yellow warning message on `stderr` without a prefix. (`npr` stands for "no prefix".)
#[macro_export]
macro_rules! warn_npr {
    ($($arg:tt)*) => {{
        use $crate::log_print_npr;
        log_print_npr!("warn", ANSI_COLOR_BOLD_YELLOW, $($arg)*);
    }};
}

/// Prints out a red error message on `stderr` without a prefix. (`npr` stands for "no prefix".)
#[macro_export]
macro_rules! error_npr {
    ($($arg:tt)*) => {{
        use $crate::log_print_npr;
        log_print_npr!("error", ANSI_COLOR_BOLD_RED, $($arg)*);
    }};
}

/// Prints out a green success message on `stderr` without a prefix. (`npr` stands for "no prefix".)
#[macro_export]
macro_rules! success_npr {
    ($($arg:tt)*) => {{
        use $crate::log_print_npr;
        log_print_npr!("success", ANSI_COLOR_BOLD_GREEN, $($arg)*);
    }};
}

/// Prints out a cyan prompt message on `stdout` without a prefix.
#[macro_export]
macro_rules! prompt_user {
    ($($arg:tt)*) => {{
        use $crate::util::terminal_colors::{ANSI_COLOR_BOLD_CYAN, ANSI_COLOR_RESET};
        use $crate::warn_npr;
        print!("{ANSI_COLOR_BOLD_CYAN}prompt:{ANSI_COLOR_RESET} {}", format!($($arg)*));
        let _ = io::stdout().flush().inspect_err(|e| warn_npr!("could not flush stdout: {e}"));
    }};
}
