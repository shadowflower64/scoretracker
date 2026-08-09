//! Common directories used by scoretracker.
use directories::ProjectDirs;
use std::path::PathBuf;

pub fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("", "shadowflower64", "scoretracker").expect("the home directory should be set before scoretracker is launched")
}

pub fn config_dir() -> PathBuf {
    project_dirs().config_local_dir().to_path_buf()
}

pub fn log_dir() -> PathBuf {
    let project_dirs = project_dirs();
    project_dirs
        .state_dir()
        .unwrap_or_else(|| project_dirs.data_local_dir())
        .join("logs")
        .to_path_buf()
}
