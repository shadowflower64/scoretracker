use crate::cmd::CmdError;
use scoretracker::config::Config;
use scoretracker::util::command_line::{ask_string, ask_yn};
use scoretracker::util::file_ex::FileEx;
use scoretracker::util::filelocked::FileLockableDataWithDefaultPath;
use scoretracker::util::lockfile;
use scoretracker::{info_npr, success_npr, warn_npr};
use std::io::{self, Write};
use std::path::PathBuf;

fn display_config(config: &Config) -> Result<(), CmdError> {
    let mut stdout = io::stdout();
    serde_json::to_writer_pretty(&stdout, &config).map_err(CmdError::ConfigSerializationError)?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}

pub fn init() -> Result<(), CmdError> {
    let path = Config::default_path();
    let has_to_confirm = match path.read_from_json() {
        Ok(Some(config)) => {
            info_npr!("current config is:");
            display_config(&config)?;
            true
        }
        Err(_) => {
            warn_npr!("current config appears to be broken or unreadable");
            true
        }
        Ok(None) => {
            info_npr!("config not found - creating new config");
            false
        }
    };

    if has_to_confirm {
        let confirm = ask_yn("are you sure you want to overwrite the current config?", None)?;
        if !confirm {
            info_npr!("not overwriting; exiting");
            return Ok(());
        }
    }

    let domain_name = ask_string("name of this device", None)?;
    let default_library_dir_path = ask_string("path to the library directory", None)?.into();
    let shared_data_repo_path = ask_string("path to the shared data repository", None)?.into();
    let config = Config {
        domain_name,
        default_library_dir_path,
        shared_data_repo_path,
    };
    path.write_as_json_pretty(&config)
        .map_err(lockfile::Error::from)
        .map_err(CmdError::ConfigWriteError)?;

    success_npr!("config written to file: {:?}", path);
    display_config(&config)
}

pub fn show() -> Result<(), CmdError> {
    let config = Config::load().map_err(CmdError::ConfigReadError)?;
    display_config(&config)
}

pub fn set(key: String, value: String) -> Result<(), CmdError> {
    let mut config = Config::lock_default_and_read(None).map_err(CmdError::ConfigReadError)?;

    match key.as_str() {
        "domain_name" => config.inner.domain_name = value,
        "shared_data_repo_path" => config.inner.shared_data_repo_path = PathBuf::from(value),
        "default_library_dir_path" => config.inner.default_library_dir_path = PathBuf::from(value),
        key => Err(CmdError::InvalidConfigKey(key.to_string()))?,
    }

    config.unlock_and_save().map_err(CmdError::ConfigWriteError)?;
    success_npr!("successfully updated config");

    Ok(())
}
