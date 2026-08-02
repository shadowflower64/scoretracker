//! Common directories used by scoretracker.
use directories::ProjectDirs;
use std::path::PathBuf;

pub fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", "shadowflower64", "scoretracker")
}

pub fn config_dir() -> PathBuf {
    project_dirs()
        .expect("could not retrieve config dirs")
        .config_local_dir()
        .to_path_buf()
}
