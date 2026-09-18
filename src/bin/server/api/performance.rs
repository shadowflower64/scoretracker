use super::super::start::AppData;
use super::ApiResult;
use actix_web::{HttpRequest, get, put, web};
use function_name::named;
use scoretracker::data::scoreboard::r#match::MatchDatabase;
use scoretracker::data::scoreboard::performance::{AnyPerformance, PerformanceDatabase};
use scoretracker::util::{filelocked::FileLockableData, uuid::UuidString};
use scoretracker::{info, log_fn_name};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// `ToSchema`-compatible wrapper for [`AnyMatch`].
// TODO: make this generate an actually useful schema
#[derive(Debug, Deserialize, ToSchema)]
pub struct AnyPerformanceWrapper {
    #[serde(flatten)]
    #[schema(ignore = true)]
    inner: Box<AnyPerformance>,
}

#[derive(Serialize)]
pub struct ListRes {
    items: Vec<Box<AnyPerformance>>,
}

#[utoipa::path(
    responses(
        (status = OK, description = "TODO")
    )
)]
#[get("/performance")]
#[named]
pub async fn list_performances(req: HttpRequest) -> ApiResult<ListRes, ()> {
    log_fn_name!(auto);
    info!("received get request for performance list");

    let app_data = req.app_data::<AppData>().expect("app data should be present");
    let performance_db = PerformanceDatabase::read_without_locking(app_data.server_config.performance_database_path())
        .expect("could not read performance database");
    ApiResult::Ok {
        result: ListRes {
            items: performance_db.performances.into_iter().map(|x| x.perf).collect::<Vec<_>>(),
        },
    }
}

#[utoipa::path(
    responses(
        (status = OK, description = "TODO")
    )
)]
#[get("/performance/{uuid}")]
#[named]
pub async fn get_performance(req: HttpRequest, path: web::Path<UuidString>) -> ApiResult<Box<AnyPerformance>, ()> {
    log_fn_name!(auto);

    let uuid = path.into_inner();
    info!("received get request for performance: {uuid}");

    let app_data = req.app_data::<AppData>().expect("app data should be present");
    let performance_db = PerformanceDatabase::read_without_locking(app_data.server_config.performance_database_path())
        .expect("could not read performance database");

    let performance = dyn_clone::clone_box(performance_db.find_performance_by_uuid(uuid).expect("performance not found"));
    let res: ApiResult<Box<AnyPerformance>, ()> = ApiResult::Ok { result: performance };
    res
}

#[utoipa::path(
    responses(
        (status = OK, description = "TODO")
    )
)]
#[put("/performance/{uuid}")]
#[named]
pub async fn put_performance(
    req: HttpRequest,
    path: web::Path<UuidString>,
    body: web::Json<AnyPerformanceWrapper>,
) -> ApiResult<Box<AnyPerformance>, ()> {
    log_fn_name!(auto);

    let uuid = path.into_inner();
    let performance = body.0.inner;
    info!("received put request for performance: {uuid} -> {performance:?}");

    assert_eq!(uuid, performance.uuid(), "uuid in url should performance uuid in request body");

    let app_data = req.app_data::<AppData>().expect("app data should be present");
    let mut match_db =
        MatchDatabase::lock_and_read(app_data.server_config.match_database_path(), None).expect("could not read match database");
    let mut performance_db = PerformanceDatabase::lock_and_read(app_data.server_config.performance_database_path(), None)
        .expect("could not read performance database");

    let response = ApiResult::Ok {
        result: performance.clone(),
    };
    performance_db
        .insert_new(performance, &match_db)
        .expect("could not insert performance into database");
    performance_db.save_and_close().expect("could not save performance database");
    response
}
