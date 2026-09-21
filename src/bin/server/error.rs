use scoretracker::{cli::cmdline_error::CmdlineError, config::toml::TomlConfigError};
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("command line error: {0}")]
    CmdlineError(#[from] CmdlineError),
    #[error("configuration error: {0}")]
    ServerConfigError(#[from] TomlConfigError),
    #[error("http server error: {0}")]
    HttpServerError(#[from] io::Error),
    #[error("postgres error: {0:?} {0}")]
    DbError(#[from] postgres::Error),
}

impl ServerError {
    pub fn exit_code_num(&self) -> u8 {
        match self {
            Self::CmdlineError(_) => 3,
            // Self::HttpServerError(..) => 40,
            // Self::ServerConfigError(..) => 40,
            _ => 40,
        }
        .into()
    }
}
