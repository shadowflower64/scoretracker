use std::{io, process::ExitCode};

use scoretracker::config::toml::TomlConfigError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("configuration error: {0}")]
    ServerConfigError(#[from] TomlConfigError),
    #[error("http server error: {0}")]
    HttpServerError(#[from] io::Error),
}

impl ServerError {
    pub fn exit_code_num(&self) -> u8 {
        match self {
            // Self::HttpServerError(..) => 40,
            // Self::ServerConfigError(..) => 40,
            _ => 40,
        }
        .into()
    }
}
