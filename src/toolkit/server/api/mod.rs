pub mod r#match;
pub mod worker_connect;

use super::start::AppData;
use super::start::DomainResolveResult;
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use function_name::named;
use scoretracker::data::library::stpl_url::StplUrl;
use scoretracker::info;
use scoretracker::log_fn_name;
use serde::{Deserialize, Serialize};
use utoipa::OpenApi;

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum ResponseGet<T: Serialize> {
    Ok(T),
    _Error,
}

fn respond_as_json(data: impl Serialize) -> HttpResponse {
    HttpResponse::Ok().body(serde_json::to_string(&data).expect("could not convert response to json"))
}

fn respond_as_json_with_status(data: impl Serialize, status: StatusCode) -> HttpResponse {
    HttpResponse::with_body(
        status,
        BoxBody::new(serde_json::to_string(&data).expect("could not convert response to json")),
    )
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum ResponseGetList<T: Serialize> {
    Ok { items: Vec<T> },
    _Error,
}

fn make_get_list_response_ok<T: Serialize>(items: Vec<T>) -> HttpResponse {
    let response = ResponseGetList::Ok { items };
    HttpResponse::Ok().body(serde_json::to_string(&response).expect("could not convert response to json"))
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum ResponsePut<T: Serialize> {
    Ok { item: T },
    _Error,
}

#[derive(Deserialize)]
pub struct ResolveStplUrlRequest {
    stpl_url: StplUrl,
}

#[utoipa::path(
    responses(
        (status = OK, description = "Domain resolved successfully", body = DomainResolveResult),
        (status = UNAUTHORIZED, description = "The requested domain exists but it requires authorization", body = DomainResolveResult),
        (status = FORBIDDEN, description = "The requested domain exists but the requester has insufficient permissions", body = DomainResolveResult),
        (status = NOT_FOUND, description = "The requested domain is not known to the server", body = DomainResolveResult)
    )
)]
#[get("/api/resolve_stpl_url")]
#[named]
pub async fn resolve_stpl_url(req: HttpRequest, q: web::Query<ResolveStplUrlRequest>) -> impl Responder {
    let stpl_url = &q.stpl_url;
    log_fn_name!(auto);
    info!("resolving url: {stpl_url}");

    let app_data = req.app_data::<AppData>().expect("app data should be present");
    let res = app_data
        .resolve_domain(&stpl_url.domain, None)
        .with_path_opt(stpl_url.path.as_deref());

    //let match_db = MatchDatabase::read_without_locking(app_data.config.match_database_path()).expect("could not read match database");
    //make_get_list_response_ok(match_db.matches)
    respond_as_json_with_status(&res, res.status_code())
}

#[derive(OpenApi)]
#[openapi(paths(resolve_stpl_url))]
pub struct ApiDoc;
