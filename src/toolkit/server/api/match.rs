use super::super::start::AppData;
use super::ResponseGetList;
use super::{ResponseGet, ResponsePut, respond_as_json};
use actix_web::{HttpRequest, Responder, get, put, web};
use function_name::named;
use scoretracker::data::scoreboard::r#match::{AnyMatch, MatchDatabase};
use scoretracker::util::{filelocked::FileLockableData, uuid::UuidString};
use scoretracker::{info, log_fn_name};
use serde::Deserialize;
use utoipa::ToSchema;

#[utoipa::path(
    responses(
        (status = OK, description = "TODO")
    )
)]
#[get("/match")]
#[named]
pub async fn get_match_list(req: HttpRequest) -> impl Responder {
    log_fn_name!(auto);
    info!("received get request for match list");

    let app_data = req.app_data::<AppData>().expect("app data should be present");
    let match_db =
        MatchDatabase::read_without_locking(app_data.server_config.match_database_path()).expect("could not read match database");
    ResponseGetList::Ok { items: match_db.matches }
}

#[utoipa::path(
    responses(
        (status = OK, description = "TODO")
    )
)]
#[get("/match/{uuid}")]
#[named]
pub async fn get_match(req: HttpRequest, path: web::Path<UuidString>) -> impl Responder {
    log_fn_name!(auto);

    let uuid = path.into_inner();
    info!("received get request for match: {uuid}");

    let app_data = req.app_data::<AppData>().expect("app data should be present");
    let match_db =
        MatchDatabase::read_without_locking(app_data.server_config.match_database_path()).expect("could not read match database");
    let match_data = match_db.find_match_by_uuid(uuid).expect("match not found");
    respond_as_json(ResponseGet::Ok(match_data))
}

/// `ToSchema`-compatible wrapper for [`AnyMatch`].
#[derive(Debug, Deserialize, ToSchema)]
pub struct AnyMatchWrapper {
    #[serde(flatten)]
    #[schema(ignore = true)]
    inner: AnyMatch,
}

#[utoipa::path(
    responses(
        (status = OK, description = "TODO")
    )
)]
#[put("/match/{uuid}")]
#[named]
pub async fn put_match(req: HttpRequest, path: web::Path<UuidString>, body: web::Json<AnyMatchWrapper>) -> impl Responder {
    log_fn_name!(auto);

    let uuid = path.into_inner();
    let match_data = body.0.inner;
    info!("received put request for match: {uuid} -> {match_data:?}");

    assert_eq!(uuid, match_data.uuid(), "uuid in url should match uuid in request body");

    let app_data = req.app_data::<AppData>().expect("app data should be present");
    let mut match_db =
        MatchDatabase::lock_and_read(app_data.server_config.match_database_path(), None).expect("could not read match database");

    let response = respond_as_json(ResponsePut::Ok { item: &match_data });
    match_db.insert(match_data).expect("could not insert match into database");
    match_db.save_and_close().expect("could not save match database");
    response
}
