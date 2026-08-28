use super::api;
use super::api::ApiDoc;
use super::api::ApiError;
use super::config::{ServerConfig, ServerConfigError};
use actix_files::NamedFile;
use actix_web::http::StatusCode;
use actix_web::{App, HttpServer, Scope, get};
use actix_web::{Error, HttpRequest};
use function_name::named;
use relative_path::{Component, RelativePathBuf};
use scoretracker::config::library_tab::InternalLibraryConnection;
use scoretracker::config::library_tab::LibraryTab;
use scoretracker::data::library::info::LibraryInfo;
use scoretracker::data::library::stpl_url::LibraryDomain;
use scoretracker::util::filelocked::FileLockableData;
use scoretracker::util::relative_path_from_segments;
use scoretracker::{debug, error, info, log_fn_name, log_should_print_debug, success, warn};
use serde::Serialize;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
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

#[derive(Debug, Clone)]
pub enum LibraryConnection {
    /// This library is on the same system as the server, and the files of the library can be directly accessed by the server process.
    Internal(InternalLibraryConnection),

    /// This library is on a different server, and direct filesystem access is not available.
    //External { address: IpAddr, port: u16 },
    External { url: String },
}

#[derive(Debug, Clone)]
pub enum AccessRules {
    DenyByDefault { allowlist: Vec<String> }, // TODO: implement authorization
    AllowByDefault { denylist: Vec<String> }, // TODO: implement authorization
}

#[derive(Debug, Clone)]
pub struct LibraryConnectionInfo {
    pub connection: LibraryConnection,
    pub library_info: LibraryInfo,
    pub permissions: AccessRules,
}

impl LibraryConnectionInfo {
    pub fn auth_required(&self) -> bool {
        todo!()
    }
    pub fn does_user_have_access(&self, _auth: UserAuth) -> bool {
        todo!()
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

pub struct UserAuth {}

impl UserAuth {}

impl AppData {
    pub fn resolve_domain(&self, domain: &LibraryDomain, auth_opt: Option<UserAuth>) -> Result<DomainResolved, DomainResolveError> {
        let lock = self.connected_libraries.read().unwrap();
        let Some(connection_info) = lock.get(domain) else {
            return Err(DomainResolveError::NotKnown);
        };

        if connection_info.auth_required() {
            let Some(auth) = auth_opt else {
                return Err(DomainResolveError::Unauthorized);
            };
            if !connection_info.does_user_have_access(auth) {
                return Err(DomainResolveError::Forbidden);
            }
        }

        match &connection_info.connection {
            LibraryConnection::External { url } => Ok(DomainResolved::External { url: url.clone() }),
            LibraryConnection::Internal { .. } => todo!(), // DomainResolveResult::Internal, // TODO: get libraryaccessapi url if it exists; do not return/expose local paths if possible
        }
    }
}

type LibraryConnections = HashMap<LibraryDomain, LibraryConnectionInfo>;

#[named]
fn connect_internal_libraries(internal_libraries: &HashMap<LibraryDomain, Vec<PathBuf>>) -> LibraryConnections {
    log_fn_name!(auto);

    let mut connections = HashMap::new();
    for (domain, paths) in internal_libraries {
        info!("connecting internal library with domain: {domain}");

        let mut loaded_library_info = None;
        let paths: Vec<_> = paths.iter().map(|library_dir| {
            let Ok(library_info) = LibraryInfo::read_without_locking(library_dir.join(LibraryInfo::STANDARD_FILENAME)) else {
                warn!("{domain}: library directory offline: {library_dir:?}, skipping");
                return (library_dir.to_owned(), Status::Unavailable);
            };

            let domain_from_info = library_info.domain.clone();
            if *domain != domain_from_info {
                error!(
                    "{domain}: installed domain name and library info domain name ({domain_from_info}) do not match; please reinstall the library"
                );
                return (library_dir.to_owned(), Status::Unavailable);
            }

            if loaded_library_info.is_none() {
                loaded_library_info = Some(library_info);
            }
            (library_dir.to_owned(), Status::Available)
        }).collect();

        let Some(library_info) = loaded_library_info else {
            error!("{domain}: no paths are available");
            continue;
        };

        let conn_info = LibraryConnectionInfo {
            connection: LibraryConnection::Internal(InternalLibraryConnection { all_paths: paths }),
            library_info: library_info,
            permissions: AccessRules::AllowByDefault { denylist: Vec::new() },
        };
        let prev_conn_info = connections.insert(domain.clone(), conn_info.clone());
        if let Some(prev_conn_info) = prev_conn_info {
            error!(
                "{domain}: library domain name collision: trying to insert {:?} but {:?} was already inserted",
                prev_conn_info, conn_info
            )
        }
    }
    connections
}

#[derive(Debug, Error)]
pub enum ServerStartError {
    #[error("configuration error: {0}")]
    ServerConfigError(#[from] ServerConfigError),
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
    let library_table = LibraryTab::load().expect("todo: error handling");
    let internal_library_connections = Arc::new(RwLock::new(connect_internal_libraries(&library_table.internal_libraries)));
    {
        let map = internal_library_connections.read().unwrap();
        if map.len() == 0 {
            warn!("no library connections established!")
        } else {
            success!("{} library connections established: {:?}", map.len(), map);
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
