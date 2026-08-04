use std::{io, process::ExitCode};

use scoretracker::{
    data::library::LibraryScanError,
    hive::worker::WorkerCreateError,
    spreadsheet::SpreadsheetImportError,
    util::{command_line::AskError, file_ex, lockfile},
};
use thiserror::Error;

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
