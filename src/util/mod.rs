use std::path::PathBuf;

pub mod cmd;
pub mod dirs;
pub mod file_ex;
pub mod filelocked;
pub mod lockfile;
pub mod log;
pub mod percentage;
pub mod terminal_colors;
pub mod timestamp;
pub mod uuid;

pub fn path_from_segments(segments: &[&str]) -> PathBuf {
    let mut path = PathBuf::new();
    for segment in segments {
        path = path.join(segment);
    }
    path
}
