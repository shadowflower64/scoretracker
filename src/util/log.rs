//! Module with simple logging macros.
use chrono::{DateTime, Local, SecondsFormat};
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
pub fn datetime_now() -> String {
    let datetime: DateTime<Local> = SystemTime::now().into();
    datetime.to_rfc3339_opts(SecondsFormat::Millis, false)
}

/// Prints out a message on `stderr`, with a function name, a thread name, and a timestamp, with the provided log level and color.
#[macro_export]
macro_rules! log_print {
    ($log_level: literal, $log_level_color: ident, $($arg:tt)*) => {{
        #[allow(unused_imports)]
        use $crate::util::terminal_colors::{ANSI_COLOR_BOLD_MAGENTA, ANSI_COLOR_BOLD_BLUE, ANSI_COLOR_BOLD_YELLOW, ANSI_COLOR_BOLD_RED, ANSI_COLOR_BOLD_GREEN, ANSI_COLOR_RESET};
        use $crate::util::log::datetime_now;
        if let Some(thread_name) = std::thread::current().name() {
            eprintln!("{} {}{}:{ANSI_COLOR_RESET} [{thread_name}/{LOG_FN_NAME}] {}", datetime_now(), $log_level_color, $log_level, format!($($arg)*));
        } else {
            eprintln!("{} {}{}:{ANSI_COLOR_RESET} [{LOG_FN_NAME}] {}", datetime_now(), $log_level_color, $log_level, format!($($arg)*));
        }
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
