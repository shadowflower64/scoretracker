use super::access_rules::AuthenticatedUser;
use super::api;
use super::api::ApiDoc;
use super::api::ApiError;
use super::config::ServerConfig;
use super::library_hall::LibraryConnections;
use actix_files::NamedFile;
use actix_web::http::StatusCode;
use actix_web::{App, HttpServer, Scope, get};
use actix_web::{Error, HttpRequest};
use function_name::named;
use relative_path::{Component, RelativePathBuf};
use scoretracker::config::library_tab::LibraryTab;
use scoretracker::config::toml::TomlConfig;
use scoretracker::config::toml::TomlConfigError;
use scoretracker::data::library::stpl_url::LibraryDomain;
use scoretracker::util::relative_path_from_segments;
use scoretracker::{debug, info, log_fn_name, log_should_print_debug, success, warn};
use serde::Serialize;
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

pub const WEB_FRONTEND_DIR_PATH_SEGMENTS: &[&str] = &["web-frontend"];
pub fn web_frontend_dir_path() -> PathBuf {
    relative_path_from_segments(WEB_FRONTEND_DIR_PATH_SEGMENTS).to_path(".")
}

pub fn static_file_dir_path() -> PathBuf {
    relative_path_from_segments(WEB_FRONTEND_DIR_PATH_SEGMENTS).join("app").to_path(".")
}

#[get("/")]
async fn index() -> Result<NamedFile, Error> {
    Ok(NamedFile::open(web_frontend_dir_path().join("app.html"))?)
}

#[get("/app/{filename:.*}")]
#[named]
async fn static_handler(req: HttpRequest) -> Result<NamedFile, Error> {
    log_fn_name!(auto);
    log_should_print_debug!(true);

    let path_str = req.match_info().query("filename");
    let relpath = RelativePathBuf::from(path_str);
    for component in relpath.components() {
        match component {
            Component::CurDir => panic!("Invalid path"),    // TODO: cleaner error?
            Component::ParentDir => panic!("Invalid path"), // TODO: cleaner error?
            Component::Normal(_) => {}
        }
    }
    let fullpath = relpath.to_path(static_file_dir_path());
    debug!("requested path: {relpath:?} -> {fullpath:?}");
    let file = NamedFile::open(fullpath)?;
    Ok(
        file.use_last_modified(true),
        /* .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }) */
    )
}

// #[get("/openapi.json")]
// #[named]
// pub async fn openapi_doc() -> impl Responder {
//     log_fn_name!(auto);
//     log_should_print_debug!(true);
//     debug!("requested openapi doc");
//     static API_DOC_JSON: LazyLock<String> = LazyLock::new(|| {
//         debug!("documenting the api...");
//         match ApiDoc::openapi().to_pretty_json() {
//             Ok(json) => json,
//             Err(e) => format!("error: {e}"),
//         }
//     });
//     HttpResponse::Ok().body(API_DOC_JSON.clone())
// }

mod testing_area {
    use actix_web::{HttpResponse, Responder, get, post};

    #[post("/test/echo")]
    pub async fn echo(req_body: String) -> impl Responder {
        HttpResponse::Ok().body(String::from("Echoed: ") + &req_body)
    }

    #[get("/test/hey")]
    pub async fn hey() -> impl Responder {
        HttpResponse::Ok().body("Hey there!")
    }
}

pub struct AppData {
    pub server_config: Arc<ServerConfig>,
    pub connected_libraries: Arc<RwLock<LibraryConnections>>,
}

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

impl AppData {
    pub fn resolve_domain(
        &self,
        domain: &LibraryDomain,
        auth_opt: Option<AuthenticatedUser>,
    ) -> Result<DomainResolved, DomainResolveError> {
        let lock = self.connected_libraries.read().unwrap();
        let Some(hall) = lock.get(domain) else {
            return Err(DomainResolveError::NotKnown);
        };

        if hall.access_rules.auth_required() {
            let Some(auth) = auth_opt else {
                return Err(DomainResolveError::Unauthorized);
            };
            if !hall.access_rules.does_user_have_access(auth) {
                return Err(DomainResolveError::Forbidden);
            }
        }

        todo!("find best mirror and return it.. alternatively, return all mirrors and let the caller deal with it")
        // match &hall {
        //     AnyLibraryConnection::External { url } => Ok(DomainResolved::External { url: url.clone() }),
        //     AnyLibraryConnection::Internal { .. } => todo!(), // DomainResolveResult::Internal, // TODO: get libraryaccessapi url if it exists; do not return/expose local paths if possible
        // }
    }
}

#[named]
fn connect_internal_libraries() -> LibraryConnections {
    let tab = LibraryTab::load().expect("todo: error handling");
    let mut connections = LibraryConnections::new();
    connections.add_internal_mirrors(tab.scan());
    connections
}

#[derive(Debug, Error)]
pub enum ServerStartError {
    #[error("configuration error: {0}")]
    ServerConfigError(#[from] TomlConfigError),
    #[error("http server error: {0}")]
    HttpServerError(#[from] io::Error),
}

#[actix_web::main]
#[named]
pub async fn server_main() -> Result<(), ServerStartError> {
    log_fn_name!(auto);
    const HOST: &str = "127.0.0.1";
    const PORT: u16 = 8080;
    info!("starting server on: http://{HOST}:{PORT}");

    let server_config = Arc::new(ServerConfig::load()?);
    let internal_library_connections = Arc::new(RwLock::new(connect_internal_libraries()));
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
            .app_data(AppData {
                server_config: Arc::clone(&server_config),
                connected_libraries: Arc::clone(&internal_library_connections),
            })
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/openapi.json", ApiDoc::openapi()))
            // .service(openapi_doc) // <- uncomment if swaggerui is not available for some reason
            .service(index)
            .service(static_handler)
            .service(testing_area::echo)
            .service(testing_area::hey)
            .service(
                Scope::new("/api")
                    .service(api::r#match::get_match_list)
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
