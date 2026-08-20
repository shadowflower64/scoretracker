use super::api;
use super::config::{ServerConfig, ServerConfigError};
use actix_files::NamedFile;
use actix_web::{App, HttpServer, get};
use actix_web::{Error, HttpRequest};
use function_name::named;
use relative_path::{Component, RelativePathBuf};
use scoretracker::config::Config;
use scoretracker::data::library::info::LibraryInfo;
use scoretracker::data::library::stpl_url::LibraryDomain;
use scoretracker::util::filelocked::FileLockableData;
use scoretracker::util::relative_path_from_segments;
use scoretracker::{debug, error, info, log_fn_name, log_should_print_debug, warn};
use serde::Serialize;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;

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

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum Status {
    Available,
    Unavailable,
}

#[derive(Debug, Clone)]
pub struct InternalLibraryConnection {
    all_paths: Vec<(PathBuf, Status)>,
}

impl InternalLibraryConnection {
    pub fn new(mut all_paths: Vec<(PathBuf, Status)>) -> Self {
        all_paths.sort_by_key(|x| x.1);
        Self { all_paths }
    }

    /// Return the first path to the library that was actually available for use when it was loaded.
    pub fn main_path(&self) -> Option<&Path> {
        self.all_paths
            .first()
            .filter(|(_, status)| *status == Status::Available)
            .map(|(path, _)| path.as_ref())
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

pub struct AppData {
    pub config: &'static Config, // TODO: this is wrong, don't use this for servers.
    pub server_config: Arc<ServerConfig>,
    pub connected_libraries: Arc<Mutex<LibraryConnections>>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum DomainResolveResult {
    /// This library lives on this server.
    Internal { url: String },
    /// This library is in another castle.
    External { url: String },
    /// No authorization to access this domain.
    Forbidden,
    /// Domain name not known.
    NotKnown,
}

impl DomainResolveResult {
    /// Append to the end of URL
    pub fn with_path(self, s: &str) -> Self {
        match self {
            Self::Internal { url } => Self::Internal { url: format!("{url}/{s}") },
            Self::External { url } => Self::External { url: format!("{url}/{s}") },
            a => a,
        }
    }
    pub fn with_path_opt(self, s: Option<&str>) -> Self {
        if let Some(a) = s { self.with_path(a) } else { self }
    }
}

pub struct UserAuth {}

impl UserAuth {
    pub fn has_access_to(&self, _conn: &LibraryConnectionInfo) -> bool {
        todo!()
    }
    pub fn guest() -> Self {
        Self {}
    }
}

impl AppData {
    pub fn resolve_domain(&self, domain: &LibraryDomain, auth: UserAuth) -> DomainResolveResult {
        if let Some(connection_info) = self.connected_libraries.lock().unwrap().get(domain) {
            if !auth.has_access_to(connection_info) {
                return DomainResolveResult::Forbidden;
            }
            match &connection_info.connection {
                LibraryConnection::External { url } => DomainResolveResult::External { url: url.clone() },
                LibraryConnection::Internal { .. } => todo!(), // DomainResolveResult::Internal,
            }
        } else {
            DomainResolveResult::NotKnown
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
    let local_library_connections: Arc<Mutex<LibraryConnections>> =
        Arc::new(Mutex::new(connect_internal_libraries(&server_config.internal_libraries)));

    Ok(HttpServer::new(move || {
        App::new()
            .app_data(AppData {
                config: Config::load().expect("todo"), // TODO: server process probably should not use the default scoretracker-toolkit config...
                server_config: Arc::clone(&server_config),
                connected_libraries: Arc::clone(&local_library_connections),
            })
            .service(index)
            .service(static_handler)
            .service(testing_area::echo)
            .service(testing_area::hey)
            .service(api::get_match_list)
            .service(api::get_match)
            .service(api::put_match)
    })
    .bind((HOST, PORT))?
    .run()
    .await?)
}
