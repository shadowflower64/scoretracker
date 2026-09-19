use actix_web::http::StatusCode;
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

use super::http::api::ApiError;

#[derive(Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainResolved {
    /// This library lives on this server.
    Internal { url: String },
    /// This library is in another castle.
    External { url: String },
}

impl DomainResolved {
    /// Append to the end of URL.
    pub fn with_path(self, s: &str) -> Self {
        match self {
            Self::Internal { url } => Self::Internal { url: format!("{url}/{s}") },
            Self::External { url } => Self::External { url: format!("{url}/{s}") },
        }
    }

    /// Append to the end of URL.
    pub fn with_path_opt(self, s: Option<&str>) -> Self {
        if let Some(a) = s { self.with_path(a) } else { self }
    }
}

#[derive(Serialize, ToSchema, Debug, Error)]
#[serde(tag = "error_kind", rename_all = "snake_case")]
pub enum DomainResolveError {
    #[error("authorization required to access this domain")]
    Unauthorized,
    #[error("no permission to access this domain")]
    Forbidden,
    #[error("domain name not known")]
    NotKnown,
}

impl ApiError for DomainResolveError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotKnown => StatusCode::NOT_FOUND,
        }
    }
}
