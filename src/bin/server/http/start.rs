use super::super::error::ServerError;

use super::super::config::ServerConfig;
use super::super::connect_internal_libraries;
use super::super::http::{index, r#static, testing_area};

use super::super::globals::ServerGlobals;
use super::api;
use super::api::ApiDoc;
use actix_web::{App, HttpServer, Scope};
use function_name::named;
use scoretracker::config::toml::{TomlConfig, TomlConfigError};
use scoretracker::util::relative_path_from_segments;
use scoretracker::{info, log_fn_name, success, warn};
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub const WEB_FRONTEND_DIR_PATH_SEGMENTS: &[&str] = &["web-frontend"];
pub fn web_frontend_dir_path() -> PathBuf {
    relative_path_from_segments(WEB_FRONTEND_DIR_PATH_SEGMENTS).to_path(".")
}
#[actix_web::main]
#[named]
pub async fn http_server_start() -> Result<(), ServerError> {
    log_fn_name!(auto);
    const HOST: &str = "127.0.0.1";
    const PORT: u16 = 8080;
    info!("starting server on: http://{HOST}:{PORT}");

    let server_config = Arc::new(ServerConfig::load()?);
    let internal_library_connections = Arc::new(RwLock::new(connect_internal_libraries::connect_internal_libraries()));
    {
        let connections = internal_library_connections.read().unwrap();
        let count = connections.len();
        if count == 0 {
            warn!("no library connections established!")
        } else {
            success!("{count} library connections established: {connections:?}");
        }
    }

    Ok(HttpServer::new(move || {
        App::new()
            .app_data(ServerGlobals {
                server_config: Arc::clone(&server_config),
                connected_libraries: Arc::clone(&internal_library_connections),
            })
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/openapi.json", ApiDoc::openapi()))
            // .service(openapi_doc) // <- uncomment if swaggerui is not available for some reason
            .service(index::index_handler)
            .service(r#static::static_handler)
            .service(testing_area::echo)
            .service(testing_area::hey)
            .service(
                Scope::new("/api")
                    .service(api::r#match::list_matches)
                    .service(api::r#match::get_match)
                    .service(api::r#match::put_match)
                    .service(api::resolve_stpl_url)
                    .service(api::worker_connect::worker_connect),
            )
    })
    .bind((HOST, PORT))?
    .run()
    .await?)
}
