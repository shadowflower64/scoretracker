use crate::data::library::stpl_url::StplUrl;
use crate::data::scoreboard::r#match::{AnyMatch, MatchDatabase};
use crate::info;
use crate::log_fn_name;
use crate::server::{AppData, UserAuth};
use crate::util::filelocked::FileLockableData;
use crate::util::uuid::UuidString;
use actix_web::{HttpRequest, HttpResponse, Responder, get, put, web};
use function_name::named;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum ResponseGet<T: Serialize> {
    Ok(T),
    _Error,
}

fn respond_as_json(data: impl Serialize) -> HttpResponse {
    HttpResponse::Ok().body(serde_json::to_string(&data).expect("could not convert response to json"))
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

#[get("/api/match")]
#[named]
pub async fn get_match_list(req: HttpRequest) -> impl Responder {
    log_fn_name!(auto);
    info!("received get request for match list");

    let app_data = req.app_data::<AppData>().expect("app data should be present");
    let match_db = MatchDatabase::read_without_locking(app_data.config.match_database_path()).expect("could not read match database");
    make_get_list_response_ok(match_db.matches)
}

#[get("/api/match/{uuid}")]
#[named]
pub async fn get_match(req: HttpRequest, path: web::Path<UuidString>) -> impl Responder {
    log_fn_name!(auto);

    let uuid = path.into_inner();
    info!("received get request for match: {uuid}");

    let app_data = req.app_data::<AppData>().expect("app data should be present");
    let match_db = MatchDatabase::read_without_locking(app_data.config.match_database_path()).expect("could not read match database");
    let match_data = match_db.find_match_by_uuid(uuid).expect("match not found");
    respond_as_json(ResponseGet::Ok(match_data))
}

#[put("/api/match/{uuid}")]
#[named]
pub async fn put_match(req: HttpRequest, path: web::Path<UuidString>, body: web::Json<AnyMatch>) -> impl Responder {
    log_fn_name!(auto);

    let uuid = path.into_inner();
    let match_data = body.into_inner();
    info!("received put request for match: {uuid} -> {match_data:?}");

    assert_eq!(uuid, match_data.uuid(), "uuid in url should match uuid in request body");

    let app_data = req.app_data::<AppData>().expect("app data should be present");
    let mut match_db = MatchDatabase::lock_and_read(app_data.config.match_database_path(), None).expect("could not read match database");

    let response = respond_as_json(ResponsePut::Ok { item: &match_data });
    match_db.insert(match_data).expect("could not insert match into database");
    match_db.save_and_close().expect("could not save match database");
    response
}

#[derive(Deserialize)]
pub struct ResolveStplUrlRequest {
    stpl_url: StplUrl,
}

#[get("/api/resolve_stpl_url")]
#[named]
pub async fn resolve_stpl_url(req: HttpRequest, q: web::Query<ResolveStplUrlRequest>) -> impl Responder {
    let stpl_url = &q.stpl_url;
    log_fn_name!(auto);
    info!("resolving url: {stpl_url}");

    let app_data = req.app_data::<AppData>().expect("app data should be present");
    let resolved = app_data
        .resolve_domain(&stpl_url.domain, UserAuth::guest())
        .with_path_opt(stpl_url.path.as_deref());

    //let match_db = MatchDatabase::read_without_locking(app_data.config.match_database_path()).expect("could not read match database");
    //make_get_list_response_ok(match_db.matches)
    respond_as_json(resolved)
}
