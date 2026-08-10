use scoretracker::util::{command_line::AskError, file_ex, lockfile};
use scoretracker::{data::library::LibraryScanError, hive::worker::WorkerCreateError, spreadsheet::SpreadsheetImportError};
use std::{io, process::ExitCode};
use thiserror::Error;
use uuid::Uuid;

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
    ConfigReadError(file_ex::Error),
    #[error("could not open config: {0}")]
    ConfigOpenError(lockfile::Error),
    #[error("could not write config: {0}")]
    ConfigWriteError(lockfile::Error),
    #[error("could not write library info: {0}")]
    LibraryInfoWriteError(file_ex::Error),
    #[error("could not open task queue: {0}")]
    TaskQueueOpenError(lockfile::Error),
    #[error("could not write task queue: {0}")]
    TaskQueueWriteError(lockfile::Error),
    #[error("could not open player database: {0}")]
    PlayerDatabaseOpenError(lockfile::Error),
    #[error("could not write player database: {0}")]
    PlayerDatabaseWriteError(lockfile::Error),
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
    #[error("could not reveal directory: {0}")]
    RevealDirectoryError(io::Error),
    #[error("player was already in database: {0}")]
    PlayerAlreadyInDatabase(Uuid),
}

impl CmdError {
    pub fn exit_status(&self) -> ExitCode {
        match self {
            #[allow(deprecated)]
            Self::UnknownError => 1,
            Self::Help => 2,
            Self::NoCommandProvided
            | Self::NoSubcommandProvided { .. }
            | Self::UnknownCommand { .. }
            | Self::UnknownSubcommand { .. }
            | Self::ArgumentNotProvided { .. } => 3,
            Self::WrongArgumentType { .. } | Self::NoGameWithId(..) | Self::InvalidConfigKey(..) => 4,
            Self::AskError(..) => 5,
            Self::IoError(..) => 6,
            Self::ConfigReadError(..) | Self::ConfigOpenError(..) => 7,
            Self::ConfigWriteError(..) => 8,
            Self::LibraryScanError(..) => 9,
            Self::WorkerCreateError(..) => 10,
            Self::ConfigSerializationError(..) => 11,
            Self::LibraryInfoWriteError(..) => 12,
            Self::DatabaseError(..) => 13,
            Self::SpreadsheetImportError(..) => 14,
            Self::TaskQueueOpenError(..) => 15,
            Self::TaskQueueWriteError(..) => 16,
            Self::RevealDirectoryError(..) => 17,
            Self::PlayerDatabaseOpenError(..) => 18,
            Self::PlayerDatabaseWriteError(..) => 19,
            Self::PlayerAlreadyInDatabase(..) => 20,
            //
            // _ => ExitCode::FAILURE,
        }
        .into()
    }
}
