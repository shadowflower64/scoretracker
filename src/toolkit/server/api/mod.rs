pub mod r#match;
pub mod worker_connect;

use super::start::AppData;
use super::start::DomainResolveError;
use super::start::DomainResolved;
use actix_web::body::EitherBody;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::web::Query;
use actix_web::{HttpRequest, HttpResponse, Responder, get};
use function_name::named;
use scoretracker::data::library::stpl_url::StplUrl;
use scoretracker::info;
use scoretracker::log_fn_name;
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::OpenApi;
use utoipa::ToSchema;

pub trait ApiError: fmt::Display + Serialize {
    /// Get HTTP status code for this result.
    fn status_code(&self) -> StatusCode;
}

#[derive(Serialize, ToSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ApiResult<T: Serialize, E: Serialize> {
    Ok {
        result: T,
    },
    OkWithStatus {
        #[serde(skip)]
        status_code: StatusCode,
        result: T,
    },
    Error {
        #[serde(skip)]
        status_code: StatusCode,
        message: String,
        #[serde(flatten)]
        error: E,
    },
}

impl<T: Serialize, E: ApiError> From<Result<T, E>> for ApiResult<T, E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(ok_data) => ApiResult::Ok { result: ok_data },
            Err(e) => {
                let status_code = e.status_code();
                let message = e.to_string();
                ApiResult::Error {
                    status_code,
                    message,
                    error: e,
                }
            }
        }
    }
}

impl<T: Serialize, E: Serialize> Responder for ApiResult<T, E> {
    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        match &self {
            Self::Ok { .. } => Json(&self).customize().with_status(StatusCode::OK).respond_to(req),
            Self::OkWithStatus { status_code, .. } | Self::Error { status_code, .. } => {
                Json(&self).customize().with_status(*status_code).respond_to(req)
            }
        }
    }
    type Body = EitherBody<EitherBody<String>>;
}

#[derive(Deserialize)]
pub struct ResolveStplUrlRequest {
    stpl_url: StplUrl,
}

pub type DomainResolveResult = ApiResult<DomainResolved, DomainResolveError>;

#[utoipa::path(
    responses(
        (status = OK, description = "Domain resolved successfully", body = DomainResolveResult),
        (status = UNAUTHORIZED, description = "The requested domain exists but it requires authorization", body = DomainResolveResult),
        (status = FORBIDDEN, description = "The requested domain exists but the requester has insufficient permissions", body = DomainResolveResult, example = json!({
            "status": "error",
            "message": "no permission to access this domain",
            "error_kind": "forbidden"
        })),
        (status = NOT_FOUND, description = "The requested domain is not known to the server", body = DomainResolveResult, example = json!({
            "status": "error",
            "message": "domain name not known",
            "error_kind": "not_known"
        }))
    )
)]
#[get("/api/resolve_stpl_url")]
#[named]
pub async fn resolve_stpl_url(
    app_data: Data<AppData>,
    Query(ResolveStplUrlRequest { stpl_url }): Query<ResolveStplUrlRequest>,
) -> DomainResolveResult {
    log_fn_name!(auto);
    info!("resolving url: {stpl_url}");

    app_data
        .resolve_domain(&stpl_url.domain, None)
        .map(|x| x.with_path_opt(stpl_url.path.as_deref()))
        .into()
}

#[derive(OpenApi)]
#[openapi(paths(resolve_stpl_url, r#match::get_match, r#match::get_match_list, r#match::put_match))]
pub struct ApiDoc;
