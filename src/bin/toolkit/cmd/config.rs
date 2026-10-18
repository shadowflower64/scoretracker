use crate::toolkit::cmd::CmdError;
use scoretracker::config::toml::TomlConfig;
use scoretracker::config::toolkit::ToolkitConfig;
use scoretracker::data::library::stpl_url::LibraryDomain;
use scoretracker::db::schema_name::SafeSchemaName;
use scoretracker::util::command_line::{ask_string, ask_yn};
use scoretracker::util::file_ex::read_from_json;
use scoretracker::{info_npr, success_npr, warn_npr};
use std::fs;
use std::io::{self, Write};

fn print_config(config: &ToolkitConfig) -> Result<(), CmdError> {
    let mut stdout = io::stdout();
    serde_json::to_writer_pretty(&stdout, &config).map_err(CmdError::ConfigSerializationError)?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}

pub fn init() -> Result<(), CmdError> {
    let path = ToolkitConfig::default_path();
    let has_to_confirm = match read_from_json(&path) {
        Ok(Some(config)) => {
            info_npr!("current config is:");
            print_config(&config)?;
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

    let default_library: LibraryDomain = ask_string("default library", None)?.try_into().expect("todo: error handling");
    let database_connection: String = ask_string("database connection string", None)?;
    let database_schema: SafeSchemaName = ask_string("database schema name", None)?.try_into().expect("todo: error handling");
    let config = ToolkitConfig {
        default_library: if default_library.to_string().is_empty() {
            None
        } else {
            Some(default_library)
        },
        database_connection: if database_connection.to_string().is_empty() {
            None
        } else {
            Some(database_connection)
        },
        database_schema: if database_schema.to_string().is_empty() {
            None
        } else {
            Some(database_schema)
        },
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| CmdError::CreateDirAllError(parent.to_owned(), e))?;
    }
    config.write(&path).map_err(CmdError::ToolkitConfigError)?;

    success_npr!("config written to file: {:?}", path);
    print_config(&config)
}

pub fn show() -> Result<(), CmdError> {
    let config = &ToolkitConfig::read_default().map_err(CmdError::ToolkitConfigError)?;
    print_config(config)
}

pub fn set(key: String, value: String) -> Result<(), CmdError> {
    let mut config = ToolkitConfig::read_default().map_err(CmdError::ToolkitConfigError)?;

    match key.as_str() {
        "default_library" => {
            config.default_library = if value.is_empty() {
                None
            } else {
                Some(value.try_into().expect("todo: error handling"))
            }
        }
        "database_connection" => config.database_connection = if value.is_empty() { None } else { Some(value) },
        "database_schema" => {
            config.database_schema = if value.is_empty() {
                None
            } else {
                Some(value.try_into().expect("todo: error handling"))
            }
        }
        key => Err(CmdError::InvalidConfigKey(key.to_string()))?,
    }

    config.write_default().map_err(CmdError::ToolkitConfigError)?;
    success_npr!("successfully updated config");

    Ok(())
}
