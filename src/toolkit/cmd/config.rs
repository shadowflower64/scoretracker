use crate::cmd::Error;
use scoretracker::config::{Config, ConfigLock};
use scoretracker::util::cmd::{ask_string, ask_yn};
use scoretracker::util::file_ex::FileEx;
use scoretracker::util::lockfile;
use scoretracker::{info_npr, success_npr, warn_npr};
use std::io::{self, Write};
use std::path::PathBuf;

fn display_config(config: &Config) -> Result<(), Error> {
    let mut stdout = io::stdout();
    serde_json::to_writer_pretty(&stdout, &config).map_err(Error::ConfigSerializationError)?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}

pub fn init() -> Result<(), Error> {
    let path = ConfigLock::env_path_or_default();
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
        .map_err(Error::ConfigWriteError)?;

    success_npr!("config written to file: {:?}", path);
    display_config(&config)
}

pub fn show() -> Result<(), Error> {
    let config = Config::load().map_err(Error::ConfigReadError)?;
    display_config(&config)
}

pub fn set(key: String, value: String) -> Result<(), Error> {
    let mut config = ConfigLock::read_default_safe(None).map_err(Error::ConfigReadError)?;

    match key.as_str() {
        "domain_name" => config.inner.domain_name = value,
        "shared_data_repo_path" => config.inner.shared_data_repo_path = PathBuf::from(value),
        "default_library_dir_path" => config.inner.default_library_dir_path = PathBuf::from(value),
        key => Err(Error::InvalidConfigKey(key.to_string()))?,
    }

    // TODO save changes in config

    config.write_to_file().map_err(Error::ConfigWriteError)?;
    success_npr!("successfully updated config");

    Ok(())
}
