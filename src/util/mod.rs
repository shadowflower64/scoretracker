use std::{ops::Neg, path::PathBuf, sync::LazyLock};

use regex::Regex;

pub mod command_line;
pub mod dirs;
pub mod file_ex;
pub mod filelocked;
pub mod lockfile;
pub mod log;
pub mod percentage;
pub mod terminal_colors;
pub mod timestamp;
pub mod uuid;

/// Create a [`PathBuf`] from individual str segments.
///
/// This function can be used to create a path with os-dependent path separators.
///
/// # Examples
/// ```
/// # use scoretracker::util::path_from_segments;
/// # use std::path::PathBuf;
/// #[cfg(target_family = "unix")]
/// assert_eq!(path_from_segments(&["directory", "file.txt"]), PathBuf::from("directory/file.txt"));
/// #[cfg(target_family = "windows")]
/// assert_eq!(path_from_segments(&["directory", "file.txt"]), PathBuf::from(r"directory\file.txt"));
/// ```
pub fn path_from_segments(segments: &[&str]) -> PathBuf {
    let mut path = PathBuf::new();
    for segment in segments {
        path = path.join(segment);
    }
    path
}

/// Parse out the YouTube video ID from a URL.
///
/// Returns [`None`] if the ID was not found.
///
/// # Examples
/// ```
/// # use scoretracker::util::youtube_id;
/// assert_eq!(youtube_id("https://youtu.be/DRi4vpCkPa0"), Some("DRi4vpCkPa0".to_string()), "test 1");
/// assert_eq!(youtube_id("https://youtu.be/DRi4vpCkPa0?si=yuArVYcjmKA3_P4e"), Some("DRi4vpCkPa0".to_string()), "test 2");
/// assert_eq!(youtube_id("https://www.youtube.com/watch?v=DRi4vpCkPa0"), Some("DRi4vpCkPa0".to_string()), "test 3");
/// assert_eq!(youtube_id("https://www.youtube.com/watch?v=DRi4vpCkPa0&feature=youtu.be"), Some("DRi4vpCkPa0".to_string()), "test 4");
/// assert_eq!(youtube_id("http://youtu.be/DRi4vpCkPa0"), Some("DRi4vpCkPa0".to_string()), "test 5");
/// assert_eq!(youtube_id("http://www.youtube.com/watch?v=DRi4vpCkPa0"), Some("DRi4vpCkPa0".to_string()), "test 6");
/// assert_eq!(youtube_id("youtu.be/DRi4vpCkPa0"), Some("DRi4vpCkPa0".to_string()), "test 7");
/// assert_eq!(youtube_id("www.youtube.com/watch?v=DRi4vpCkPa0"), Some("DRi4vpCkPa0".to_string()), "test 8");
/// assert_eq!(youtube_id("youtube.com/watch?v=DRi4vpCkPa0"), Some("DRi4vpCkPa0".to_string()), "test 9");
/// assert_eq!(youtube_id("DRi4vpCkPa0"), Some("DRi4vpCkPa0".to_string()), "test 10");
/// ```
pub fn youtube_id(url: &str) -> Option<String> {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(?:(?:https?:\/\/)?(?:(?:(?:www.)?youtube\.com\/watch\?v=)|(?:youtu\.be\/)))?([a-zA-Z0-9-=]{11})(?:[#&?]|$).*")
            .expect("could not parse regex")
    });
    let captures = REGEX.captures(url)?;
    let capture_match = captures.get(1)?.as_str().to_owned();
    Some(capture_match)
}

/// Normalizes an [`u64`] (or any value greater than or equal to 0) to the range [0..1).
pub fn normalize_unsigned_to_unit_range(uint: u64) -> f64 {
    let x = uint as f64 + 1.0; // in range [1..inf), inf is the highest value
    let x = x.recip(); // in range (0..1], 0 is the highest value
    let x = x.neg(); // in range [-1..0), 0 is the highest value
    let x = x + 1.0; // in range [0..1), 1 is the highest value
    x
}
