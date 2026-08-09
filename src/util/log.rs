//! Module with simple logging macros.
use crate::util::terminal_colors::{ANSI_COLOR_BOLD_RED, ANSI_COLOR_RESET};
use chrono::{DateTime, Local, SecondsFormat};
use smol::io;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::SystemTime;

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
    ($arg:expr) => {
        pub const PRINT_DEBUG_MESSAGES: bool = $arg;
    };
}

/// Returns the current date and time as a string. Used by logging macros.
pub fn create_log_filename(worker: Option<(&str, u32)>) -> String {
    let datetime: DateTime<Local> = SystemTime::now().into();
    let datetime = datetime.format("%Y_%m_%d_%H_%M_%S");
    if let Some((short_name, pid)) = worker {
        format!("log_{datetime}_{}-{pid}_worker.log", short_name.to_lowercase())
    } else {
        format!("log_{datetime}.log")
    }
}

static LOG_FILE: LazyLock<Mutex<Option<File>>> = LazyLock::new(|| Mutex::new(None));

pub fn open_log_file(path: &Path) -> Result<(), (io::Error, PathBuf, bool)> {
    let log_file_parent = path.parent().expect("the log file path should have a parent path");
    fs::create_dir_all(log_file_parent).map_err(|e| (e, path.to_owned(), false))?;
    let file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .map_err(|e| (e, path.to_owned(), true))?;
    *LOG_FILE.lock().unwrap() = Some(file);
    Ok(())
}

/// Prints out a message on `stderr`, with a function name, a thread name, and a timestamp, with the provided log level and color.
/// Also prints out a non-colored message to the log file in [`LOG_FILE`].
pub fn on_log<F: Fn() -> String, G: Fn() -> String>(fmt_plain: F, fmt_colored: G) {
    eprintln!("{}", fmt_colored());

    let mut log_file = LOG_FILE.lock().unwrap();
    if let Some(file) = &mut *log_file {
        if let Err(e) = writeln!(file, "{}", fmt_plain()) {
            eprintln!("{ANSI_COLOR_BOLD_RED}could not write to log file: {e}{ANSI_COLOR_RESET}");
        }
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

/// Prints out a magenta debug message on `stderr`, with a prefix containing the function name, a thread name, and a timestamp.
///
/// This log message is printed only if the `PRINT_DEBUG_MESSAGES` flag (set using the [`log_should_print_debug!`] macro) is set to true.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        if PRINT_DEBUG_MESSAGES {
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

#[macro_export]
macro_rules! log_print_npr {
    ($log_level: literal, $log_level_color: ident, $($arg:tt)*) => {{
        #[allow(unused_imports)]
        use $crate::util::terminal_colors::{ANSI_COLOR_BOLD_MAGENTA, ANSI_COLOR_BOLD_BLUE, ANSI_COLOR_BOLD_YELLOW, ANSI_COLOR_BOLD_RED, ANSI_COLOR_BOLD_GREEN, ANSI_COLOR_BOLD_CYAN, ANSI_COLOR_RESET};
        eprintln!("{}{}:{ANSI_COLOR_RESET} {}", $log_level_color, $log_level, format!($($arg)*));
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
