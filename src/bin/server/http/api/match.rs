use crate::server::globals::ServerGlobals;
use crate::server::http::api::ApiResult;
use actix_web::{HttpRequest, get, put, web};
use chrono::Utc;
use function_name::named;
use scoretracker::data::scoreboard::r#match::AnyMatchDetails;
use scoretracker::{
    info, log_fn_name,
    util::{filelocked::FileLockableData, uuid::UuidString},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// `ToSchema`-compatible wrapper for [`AnyMatch`].
// TODO: make this generate an actually useful schema
#[derive(Debug, Deserialize, ToSchema)]
pub struct AnyMatchWrapper {
    #[serde(flatten)]
    #[schema(ignore = true)]
    inner: Box<AnyMatchDetails>,
}

#[derive(Serialize)]
pub struct ListRes {
    items: Vec<Box<AnyMatchDetails>>,
}

#[utoipa::path(
    context_path = "/api",
    responses(
        (status = OK, description = "TODO")
    )
)]
#[get("/match")]
#[named]
pub async fn list_matches(req: HttpRequest) -> ApiResult<ListRes, ()> {
    log_fn_name!(auto);
    info!("received get request for match list");

    let globals = req.app_data::<ServerGlobals>().expect("app data should be present");
    let db = globals.db.lock().await;

    for row in db
        .query(
            "SELECT match_uuid, timestamp, game, chartset_id, proof, details, metadata FROM matches",
            &[],
        )
        .await
        .expect("sql statement could not be executed")
    {
        let match_uuid: Uuid = row.get("match_uuid");
        let timestamp: chrono::DateTime<Utc> = row.get("timestamp");
        let game: String = row.get("game");
        let chartset_id: String = row.get("chartset_id");
        let proof: Vec<Uuid> = row.get("proof");
        let details: serde_json::Value = row.get("details");
        let metadata: serde_json::Value = row.get("metadata");
        println!("match_uuid: {match_uuid}, chartset_id: {chartset_id}")
    }
    todo!()
    // TODO: remove the thing below, use the real db
    // let match_db = MatchDatabase::read_without_locking(globals.server_config.match_database_path()).expect("could not read match database");

    // ApiResult::Ok {
    //     result: ListRes { items: match_db.matches },
    // }
}

#[utoipa::path(
    context_path = "/api",
    responses(
        (status = OK, description = "TODO")
    )
)]
#[get("/match/{uuid}")]
#[named]
pub async fn get_match(req: HttpRequest, path: web::Path<UuidString>) -> ApiResult<Box<AnyMatchDetails>, ()> {
    log_fn_name!(auto);

    let uuid = path.into_inner();
    info!("received get request for match: {uuid}");
    todo!()

    // let app_data = req.app_data::<ServerGlobals>().expect("app data should be present");
    // let match_db =
    //     MatchDatabase::read_without_locking(app_data.server_config.match_database_path()).expect("could not read match database");

    // let match_data = dyn_clone::clone_box(match_db.find_match_by_uuid(uuid).expect("match not found"));
    // let res: ApiResult<Box<AnyMatchDetails>, ()> = ApiResult::Ok { result: match_data };
    // res
}

#[utoipa::path(
    context_path = "/api",
    responses(
        (status = OK, description = "TODO")
    )
)]
#[put("/match/{uuid}")]
#[named]
pub async fn put_match(
    req: HttpRequest,
    path: web::Path<UuidString>,
    body: web::Json<AnyMatchWrapper>,
) -> ApiResult<Box<AnyMatchDetails>, ()> {
    log_fn_name!(auto);

    let uuid = path.into_inner();
    let match_data = body.0.inner;
    info!("received put request for match: {uuid} -> {match_data:?}");
    todo!();

    // assert_eq!(uuid, match_data.uuid(), "uuid in url should match uuid in request body");

    // let app_data = req.app_data::<ServerGlobals>().expect("app data should be present");
    // let mut match_db =
    //     MatchDatabase::lock_and_read(app_data.server_config.match_database_path(), None).expect("could not read match database");

    // let response = ApiResult::Ok {
    //     result: match_data.clone(),
    // };
    // match_db.insert_new(match_data).expect("could not insert match into database");
    // match_db.save_and_close().expect("could not save match database");
    // response
}
