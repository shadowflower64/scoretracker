use crate::cmd::{self};
use scoretracker::config::Config;
use scoretracker::data::library::LibraryScanError;
use scoretracker::data::library::stpl_url::{LibraryDomain, LibraryDomainName};
use scoretracker::hive::worker::WorkerCreateError;
use scoretracker::info_npr;
use scoretracker::spreadsheet::SpreadsheetImportError;
use scoretracker::util::command_line::AskError;
use scoretracker::util::{file_ex, lockfile};
use std::io::{self};
use std::path::PathBuf;
use std::process::ExitCode;
use thiserror::Error;

pub mod automark;
pub mod config;
pub mod hive;
pub mod library;
pub mod performance;
pub mod player;
pub mod spreadsheet;
pub mod vitals;

#[derive(Debug, Error)]
pub enum CmdError {
    #[error("unknown error")]
    #[deprecated = "this should not be used, please define an actual error variant instead"]
    UnknownError,
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
    #[error("could not write library info: {0}")]
    LibraryInfoWriteError(file_ex::Error),
    #[error("could not serialize config: {0}")]
    ConfigSerializationError(serde_json::Error),
    #[error("invalid config key: {0}")]
    InvalidConfigKey(String),
    #[error("library scan error: {0}")]
    LibraryScanError(#[from] LibraryScanError),
    #[error("database read/write error: {0}")]
    DatabaseError(lockfile::Error),
    #[error("could not create worker: {0}")]
    WorkerCreateError(#[from] WorkerCreateError),
    #[error("no game with id: {0}")]
    NoGameWithId(String),
    #[error("spreadsheet import error: {0}")]
    SpreadsheetImportError(#[from] SpreadsheetImportError),
}

impl CmdError {
    pub fn exit_status(&self) -> ExitCode {
        match self {
            #[allow(deprecated)]
            Self::UnknownError => ExitCode::from(1),
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
            Self::LibraryScanError(..) => ExitCode::from(9),
            Self::WorkerCreateError(..) => ExitCode::from(10),
            Self::ConfigSerializationError(..) => ExitCode::from(11),
            Self::LibraryInfoWriteError(..) => ExitCode::from(12),
            Self::DatabaseError(..) => ExitCode::from(13),
            Self::SpreadsheetImportError(..) => ExitCode::from(14),
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
            Some(unknown_command) => Err(CmdError::UnknownCommand {
                cmd: unknown_command.to_string()
            }),
            None => Err(CmdError::NoCommandProvided),
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
            Some(unknown_subcommand) => Err(CmdError::UnknownSubcommand{
                cmd: $full_command_name.clone(),
                subcmd: unknown_subcommand.to_string()
            }),
            None => Err(CmdError::NoSubcommandProvided{
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
    ($x: ident, $arg_value: ident) => {
        std::convert::TryInto::<$x>::try_into($arg_value)
    };
}

macro_rules! arg {
    ($arg_name: ident : $arg_type: tt, $description: literal, $full_command_name: ident, $args: ident, $arg_num: literal) => {
        let $arg_name = $args.get($arg_num).ok_or(CmdError::ArgumentNotProvided {
            cmd: $full_command_name.clone(),
            arg_name: stringify!($arg_name).to_string(),
            arg_desc: $description.to_string(),
        })?;
        let $arg_name = parse_arg!($arg_type, $arg_name).map_err(|e| CmdError::WrongArgumentType {
            cmd: $full_command_name.clone(),
            arg_name: stringify!($arg_name).to_string(),
            arg_desc: $description.to_string(),
            arg_type: stringify!($arg_type).to_string(),
            err_msg: format!("{}", e),
        })?;
    };
}

#[allow(unused_assignments)]
pub fn handle_command(args: &[String]) -> Result<(), CmdError> {
    type E = CmdError;
    // fcn - full command name
    let mut fcn;
    cmd!(root fcn, args, 1,
        "hello" => {
            info_npr!("hello world!");
            Ok(())
        },
        "spreadsheet" => cmd!(fcn, args, 2,
            "import-org-ods" => {
                arg!(path: PathBuf, "path of the ods spreadsheet file", fcn, args, 3);
                cmd::spreadsheet::import_org_ods(&path)
            },
            "import-org-xlsx" => {
                arg!(path: PathBuf, "path of the xlsx spreadsheet file", fcn, args, 3);
                cmd::spreadsheet::import_org_xlsx(&path)
            },
        ),
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
            "init" => {
                arg!(library_dir: PathBuf, "path of the library directory", fcn, args, 3);
                arg!(library_domain_name: LibraryDomainName, "library domain name", fcn, args, 4);
                cmd::library::init(&library_dir, library_domain_name)
            },
            "rescan" => {
                arg!(library_dir: PathBuf, "path of the library directory", fcn, args, 3);
                cmd::library::rescan(&library_dir).map_err(E::LibraryScanError)
            },
            "rescan-default" => {
                let config = Config::load().map_err(CmdError::ConfigReadError)?;
                cmd::library::rescan(&config.default_library_dir_path).map_err(E::LibraryScanError)
            },
            "remove-domain" => {
                arg!(library_domain: LibraryDomain, "library domain name", fcn, args, 3);
                cmd::library::remove_domain(library_domain).map_err(E::DatabaseError)
            }
        ),
        "hive" => cmd!(fcn, args, 2,
            "worker" => cmd!(fcn, args, 3,
                "spawn" => {
                    arg!(persistent: bool, "should the worker stay alive after finishing a task?", fcn, args, 4);
                    cmd::hive::spawn_worker(persistent)
                }
            ),
            "task" => cmd!(fcn, args, 3,
                "add" => {
                    cmd::hive::add_task()
                }
            )
        ),
        "vitals" => cmd!(fcn, args, 2,
            "all" => {
                cmd::vitals::check_all()
            }
        ),
        "performance" => cmd!(fcn, args, 2,
            "add" => {
                arg!(game_id: String, "id of the game to add a performance for", fcn, args, 3);
                cmd::performance::add(game_id)
            }
        ),
        "player" => cmd!(fcn, args, 2,
            "add" => {
                arg!(name: String, "name of the player", fcn, args, 3);
                cmd::player::add(name)
            }
        )
    )
}
