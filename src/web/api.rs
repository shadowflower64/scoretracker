use crate::data::scoreboard::r#match::{AnyMatch, MatchDatabase};
use crate::info;
use crate::log_fn_name;
use crate::util::filelocked::FileLockableData;
use crate::util::uuid::UuidString;
use crate::web::AppData;
use actix_web::{HttpRequest, HttpResponse, Responder, get, put, web};
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum ResponseGet<T: Serialize> {
    Ok { item: T },
    _Error,
}

fn make_get_response_ok<T: Serialize>(item: &T) -> HttpResponse {
    let response = ResponseGet::Ok { item };
    HttpResponse::Ok().body(serde_json::to_string(&response).expect("could not convert response to json"))
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

fn make_put_response_ok<T: Serialize>(item: &T) -> HttpResponse {
    let response = ResponsePut::Ok { item };
    HttpResponse::Ok().body(serde_json::to_string(&response).expect("could not convert response to json"))
}

#[get("/api/match")]
pub async fn get_match_list(req: HttpRequest) -> impl Responder {
    log_fn_name!("get_match_list");
    info!("received get request for match list");

    let app_data = req.app_data::<AppData>().expect("app data should be present");
    let match_db = MatchDatabase::read_without_locking(app_data.config.match_database_path()).expect("could not read match database");
    make_get_list_response_ok(match_db.matches)
}

#[get("/api/match/{uuid}")]
pub async fn get_match(req: HttpRequest, path: web::Path<UuidString>) -> impl Responder {
    log_fn_name!("get_match");

    let uuid = path.into_inner();
    info!("received get request for match: {uuid}");

    let app_data = req.app_data::<AppData>().expect("app data should be present");
    let match_db = MatchDatabase::read_without_locking(app_data.config.match_database_path()).expect("could not read match database");
    let match_data = match_db.find_match_by_uuid(uuid).expect("match not found");
    make_get_response_ok(&match_data)
}

#[put("/api/match/{uuid}")]
pub async fn put_match(req: HttpRequest, path: web::Path<UuidString>, body: web::Json<AnyMatch>) -> impl Responder {
    log_fn_name!("put_match");

    let uuid = path.into_inner();
    let match_data = body.into_inner();
    info!("received put request for match: {uuid} -> {match_data:?}");

    assert_eq!(uuid, match_data.uuid(), "uuid in url should match uuid in request body");

    let app_data = req.app_data::<AppData>().expect("app data should be present");
    let mut match_db = MatchDatabase::lock_and_read(app_data.config.match_database_path(), None).expect("could not read match database");

    let res = make_put_response_ok(&match_data);
    match_db.insert(match_data).expect("could not insert match into database");
    match_db.save_and_close().expect("could not save match database");
    res
}
