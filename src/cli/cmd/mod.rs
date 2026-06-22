use crate::cmd::{self, library::LibraryScanError};
use scoretracker::config::Config;
use scoretracker::hive::worker::WorkerCreateError;
use scoretracker::info_npr;
use scoretracker::util::cmd::AskError;
use scoretracker::util::lockfile;
use std::io::{self};
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

pub mod automark;
pub mod config;
pub mod hive;
pub mod library;
pub mod performance;

#[derive(Debug, Error)]
pub enum Error {
    #[error("help was displayed")]
    Help,
    #[error("no command provided")]
    NoCommandProvided,
    #[error("{cmd}: no subcommand provided")]
    NoSubcommandProvided { cmd: String },
    #[error("unknown command: {cmd}")]
    UnknownCommand { cmd: String },
    #[error("{cmd}: unknown subcommand: {subcmd}")]
    UnknownSubcommand { cmd: String, subcmd: String },
    #[error("{cmd}: argument not provided: {arg_name} ({arg_desc})")]
    ArgumentNotProvided { cmd: String, arg_name: String, arg_desc: String },
    #[error("{cmd}: argument '{arg_name}' could not be converted to {arg_type}: {err_msg}")]
    WrongArgumentType {
        cmd: String,
        arg_name: String,
        arg_desc: String,
        arg_type: String,
        err_msg: String,
    },
    #[error("generic input/output error: {0}")]
    IoError(#[from] io::Error),
    #[error("could not get string from stdin: {0}")]
    AskError(#[from] AskError),
    #[error("could not read config: {0}")]
    ConfigReadError(lockfile::Error),
    #[error("could not write config: {0}")]
    ConfigWriteError(lockfile::Error),
    #[error("could not serialize config: {0}")]
    ConfigSerializationError(serde_json::Error),
    #[error("invalid config key: {0}")]
    InvalidConfigKey(String),
    #[error("library rescan error: {0}")]
    LibraryRescanError(#[from] LibraryScanError),
    #[error("could not create worker: {0}")]
    WorkerCreateError(#[from] WorkerCreateError),
    #[error("no game with id: {0}")]
    NoGameWithId(String),
}

impl Error {
    pub fn exit_status(&self) -> ExitCode {
        match self {
            Self::Help => ExitCode::from(2),
            Self::NoCommandProvided
            | Self::NoSubcommandProvided { .. }
            | Self::UnknownCommand { .. }
            | Self::UnknownSubcommand { .. }
            | Self::ArgumentNotProvided { .. } => ExitCode::from(3),
            Self::WrongArgumentType { .. } | Self::NoGameWithId(..) | Self::InvalidConfigKey(..) => ExitCode::from(4),
            Self::AskError(..) => ExitCode::from(5),
            Self::IoError(..) => ExitCode::from(6),
            Self::ConfigReadError(..) => ExitCode::from(7),
            Self::ConfigWriteError(..) => ExitCode::from(8),
            Self::LibraryRescanError(..) => ExitCode::from(9),
            Self::WorkerCreateError(..) => ExitCode::from(10),
            Self::ConfigSerializationError(..) => ExitCode::from(11),
            // _ => ExitCode::FAILURE,
        }
    }
}

// These macros are probably not the best idea ever but it works well for now so i'm keeping it for the time being
macro_rules! cmd {
    (root $full_command_name: ident, $args: ident, $arg_num: literal, $($command_name:literal => $code:stmt),+ $(,)?) => {{
        $full_command_name = String::new();
        match $args.get($arg_num).map(String::as_str) {
            $(
                Some($command_name) => {
                    const COMMAND_NAME: &str = $command_name;
                    $full_command_name = COMMAND_NAME.to_string();
                    $code
                }
            )+
            Some(unknown_command) => Err(Error::UnknownCommand {
                cmd: unknown_command.to_string()
            }),
            None => Err(Error::NoCommandProvided),
        }
    }};
    ($full_command_name: ident, $args: ident, $arg_num: literal, $($subcommand_name:literal => $code:stmt),+ $(,)?) => {{
        match $args.get($arg_num).map(String::as_str) {
            $(
                Some($subcommand_name) => {
                    const COMMAND_NAME: &str = $subcommand_name;
                    $full_command_name = format!("{}:{}", $full_command_name, COMMAND_NAME);
                    $code
                }
            )+
            Some(unknown_subcommand) => Err(Error::UnknownSubcommand{
                cmd: $full_command_name.clone(),
                subcmd: unknown_subcommand.to_string()
            }),
            None => Err(Error::NoSubcommandProvided{
                cmd: $full_command_name.clone()
            }),
        }
    }};
}

macro_rules! parse_arg {
    (bool, $arg_value: ident) => {
        $arg_value.parse::<bool>()
    };
    (u64, $arg_value: ident) => {
        $arg_value.parse::<u64>()
    };
    (i64, $arg_value: ident) => {
        $arg_value.parse::<i64>()
    };
    (f64, $arg_value: ident) => {
        $arg_value.parse::<f64>()
    };
    (String, $arg_value: ident) => {
        Result::<String, std::convert::Infallible>::Ok($arg_value.to_string())
    };
    (PathBuf, $arg_value: ident) => {
        std::convert::TryInto::<PathBuf>::try_into($arg_value)
    };
    (Uuid, $arg_value: ident) => {
        std::convert::TryInto::<Uuid>::try_into($arg_value)
    };
}

macro_rules! arg {
    ($arg_name: ident : $arg_type: tt, $description: literal, $full_command_name: ident, $args: ident, $arg_num: literal) => {
        let $arg_name = $args.get($arg_num).ok_or(Error::ArgumentNotProvided {
            cmd: $full_command_name.clone(),
            arg_name: stringify!($arg_name).to_string(),
            arg_desc: $description.to_string(),
        })?;
        let $arg_name = parse_arg!($arg_type, $arg_name).map_err(|e| Error::WrongArgumentType {
            cmd: $full_command_name.clone(),
            arg_name: stringify!($arg_name).to_string(),
            arg_desc: $description.to_string(),
            arg_type: stringify!($arg_type).to_string(),
            err_msg: format!("{}", e),
        })?;
    };
}

#[allow(unused_assignments)]
pub fn handle_command(args: &[String]) -> Result<(), Error> {
    type E = Error;
    // fcn - full command name
    let mut fcn;
    cmd!(root fcn, args, 1,
        "hello" => {
            info_npr!("hello world!");
            Ok(())
        },
        "config" => cmd!(fcn, args, 2,
            "init" => {
                cmd::config::init()
            },
            "show" => {
                cmd::config::show()
            },
            "set" => {
                arg!(config_key: String, "name of the key to change in the configuration", fcn, args, 3);
                arg!(config_value: String, "new value for the selected key", fcn, args, 4);
                cmd::config::set(config_key, config_value)
            }
        ),
        "library" => cmd!(fcn, args, 2,
            "rescan" => {
                arg!(path: PathBuf, "path of the library directory", fcn, args, 3);
                cmd::library::rescan(&path).map_err(E::LibraryRescanError)
            },
            "rescan-default" => {
                let config = Config::load().map_err(Error::ConfigReadError)?;
                cmd::library::rescan(&config.default_library_dir_path).map_err(E::LibraryRescanError)
            }
        ),
        "hive" => cmd!(fcn, args, 2,
            "worker" => cmd!(fcn, args, 3,
                "spawn" => {
                    arg!(persistent: bool, "should the worker stay alive after finishing a task?", fcn, args, 4);
                    cmd::hive::spawn_worker(persistent)
                }
            )
        ),
        "performance" => cmd!(fcn, args, 2,
            "add" => {
                arg!(game_id: String, "id of the game to add a performance for", fcn, args, 3);
                cmd::performance::add(game_id)
            }
        )
    )
}
