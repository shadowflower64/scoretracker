use scoretracker::util::{command_line::AskError, file_ex, lockfile};
use scoretracker::{data::library::LibraryScanError, hive::worker::WorkerCreateError, spreadsheet::SpreadsheetImportError};
use std::path::PathBuf;
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
    #[error("no game with id: {0}")]
    NoGameWithId(String),
    #[error("invalid config key: {0}")]
    InvalidConfigKey(String),
    //
    #[error("could not get string from stdin: {0}")]
    AskError(#[from] AskError),
    #[error("generic input/output error: {0}")]
    IoError(#[from] io::Error),
    // ---
    #[error("could not read config: {0}")]
    ConfigReadError(&'static file_ex::Error),
    #[error("could not open config: {0}")]
    ConfigOpenError(lockfile::Error),
    #[error("could not write config: {0}")]
    ConfigWriteError(lockfile::Error),
    //
    #[error("could not read library info: {0}")]
    LibraryInfoReadError(file_ex::Error),
    #[error("could not write library info: {0}")]
    LibraryInfoWriteError(file_ex::Error),
    //
    #[error("could not read library index: {0}")]
    LibraryIndexReadError(file_ex::Error),
    #[error("could not write library index: {0}")]
    LibraryIndexWriteError(file_ex::Error),
    //
    #[error("could not open library database: {0}")]
    LibraryDatabaseOpenError(lockfile::Error),
    #[error("could not write library database: {0}")]
    LibraryDatabaseWriteError(lockfile::Error),
    //
    #[error("could not open task queue: {0}")]
    TaskQueueOpenError(lockfile::Error),
    #[error("could not write task queue: {0}")]
    TaskQueueWriteError(lockfile::Error),
    //
    #[error("could not open player database: {0}")]
    PlayerDatabaseOpenError(lockfile::Error),
    #[error("could not write player database: {0}")]
    PlayerDatabaseWriteError(lockfile::Error),
    //
    #[error("could not open match database: {0}")]
    MatchDatabaseOpenError(lockfile::Error),
    #[error("could not write match database: {0}")]
    MatchDatabaseWriteError(lockfile::Error),
    //
    #[error("could not open performance database: {0}")]
    PerformanceDatabaseOpenError(lockfile::Error),
    #[error("could not write performance database: {0}")]
    PerformanceDatabaseWriteError(lockfile::Error),
    // ---
    #[error("library scan error: {0}")]
    LibraryScanError(#[from] LibraryScanError),
    #[error("could not create worker: {0}")]
    WorkerCreateError(#[from] WorkerCreateError),
    #[error("spreadsheet import error: {0}")]
    SpreadsheetImportError(#[from] SpreadsheetImportError),
    #[error("could not reveal directory: {0}")]
    RevealDirectoryError(io::Error),
    #[error("player was already in database: {0}")]
    PlayerAlreadyInDatabase(Uuid),
    #[error("could not serialize config: {0}")]
    ConfigSerializationError(serde_json::Error),
    #[error("could not create directory: {0}; reason: {1}")]
    CreateDirAllError(PathBuf, io::Error),
    // ---
    #[error("library needs to be rescanned: uuid not found in library database: {0}")]
    LibraryRescanNeeded(Uuid),
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
            // ---
            Self::ConfigReadError(..) | Self::ConfigOpenError(..) => 11,
            Self::ConfigWriteError(..) => 12,
            Self::LibraryInfoReadError(..) => 13,
            Self::LibraryInfoWriteError(..) => 14,
            Self::LibraryIndexReadError(..) => 15,
            Self::LibraryIndexWriteError(..) => 16,
            Self::LibraryDatabaseOpenError(..) => 17,
            Self::LibraryDatabaseWriteError(..) => 18,
            Self::TaskQueueOpenError(..) => 19,
            Self::TaskQueueWriteError(..) => 20,
            Self::PlayerDatabaseOpenError(..) => 21,
            Self::PlayerDatabaseWriteError(..) => 22,
            Self::MatchDatabaseOpenError(..) => 23,
            Self::MatchDatabaseWriteError(..) => 24,
            Self::PerformanceDatabaseOpenError(..) => 25,
            Self::PerformanceDatabaseWriteError(..) => 26,
            // ---
            Self::LibraryScanError(..) => 31,
            Self::WorkerCreateError(..) => 32,
            Self::SpreadsheetImportError(..) => 33,
            Self::RevealDirectoryError(..) => 34,
            Self::PlayerAlreadyInDatabase(..) => 35,
            Self::ConfigSerializationError(..) => 36,
            Self::CreateDirAllError(..) => 37,
            // ---
            Self::LibraryRescanNeeded(..) => 51,
        }
        .into()
    }
}
