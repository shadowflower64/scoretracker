use crate::server::{
    config::ServerConfig,
    connect_internal_libraries,
    error::ServerError,
    globals::ServerGlobals,
    http::{
        api::{self, ApiDoc},
        index, r#static, testing_area,
    },
};
use actix_web::{App, HttpServer, Scope};
use function_name::named;
use scoretracker::{config::toml::TomlConfig, db::Database, info, log_fn_name, success, util::relative_path_from_segments, warn};
use smol::lock::Mutex;
use std::{
    path::PathBuf,
    str::FromStr,
    sync::{Arc, RwLock},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub const WEB_FRONTEND_DIR_PATH_SEGMENTS: &[&str] = &["web-frontend"];
pub fn web_frontend_dir_path() -> PathBuf {
    relative_path_from_segments(WEB_FRONTEND_DIR_PATH_SEGMENTS).to_path(".")
}

// #[named]
// async fn connect_to_db(database_connection_string: &str) -> Result<tokio_postgres::Client, tokio_postgres::Error> {
//     log_fn_name!(auto);

//     let config = tokio_postgres::Config::from_str(database_connection_string)?;
//     // info!("config: {config:?}");

//     let (client, connection) = config.connect(tokio_postgres::NoTls).await?;
//     actix_web::rt::spawn(async move {
//         if let Err(e) = connection.await {
//             eprintln!("connection error: {}", e);
//         }
//     });

//     success!("connected to database");
//     Ok(client)
// }

#[tokio::main]
#[named]
pub async fn http_server_start_runtime() -> Result<(), ServerError> {
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

    info!("connecting to database...");
    let db = Arc::new(Mutex::new(
        Database::connect_with_tokio(&server_config.database_connection, &server_config.database_schema).await?,
    ));
    success!("connected to database successfully");

    info!("starting actual http server...");
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(ServerGlobals {
                server_config: Arc::clone(&server_config),
                connected_libraries: Arc::clone(&internal_library_connections),
                db: Arc::clone(&db),
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
