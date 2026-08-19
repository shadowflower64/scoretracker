pub mod api;
pub mod config;

use crate::config::Config;
use crate::data::library::info::LibraryInfo;
use crate::data::library::stpl_url::LibraryDomainName;
use crate::server::config::ServerConfig;
use crate::util::filelocked::FileLockableData;
use crate::util::relative_path_from_segments;
use crate::{debug, error, info, log_fn_name, log_should_print_debug};
use actix_files::NamedFile;
use actix_web::{App, HttpServer, get};
use actix_web::{Error, HttpRequest};
use relative_path::{Component, RelativePathBuf};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};

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
async fn static_handler(req: HttpRequest) -> Result<NamedFile, Error> {
    log_fn_name!("static_handler");
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

mod test {
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
    Internal { path: PathBuf },

    /// This library is on a different server, and direct filesystem access is not available.
    //External { address: IpAddr, port: u16 },
    External { url: String },
}

#[derive(Debug, Clone)]
pub enum AccessRules {
    DenyByDefault { allowlist: Vec<String> },
    AllowByDefault { denylist: Vec<String> },
}

#[derive(Debug, Clone)]
pub struct LibraryConnectionInfo {
    pub connection: LibraryConnection,
    pub library_info: LibraryInfo,
    pub permissions: AccessRules,
}

pub struct AppData {
    pub config: &'static Config, // TODO: this is wrong, don't use this for servers.
    pub server_config: &'static ServerConfig,
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
    pub fn resolve_domain(&self, domain: &LibraryDomainName, auth: UserAuth) -> DomainResolveResult {
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

type LibraryConnections = HashMap<LibraryDomainName, LibraryConnectionInfo>;

fn connect_local_libraries(library_dirs: &[PathBuf]) -> LibraryConnections {
    log_fn_name!("connect_local_libraries");

    let mut connections = HashMap::new();
    for library_dir in library_dirs {
        info!("connecting local library at: {library_dir:?}");
        let library_info = LibraryInfo::read_without_locking(library_dir.join(LibraryInfo::STANDARD_FILENAME)).expect("todo");
        let domain = library_info.domain.clone();
        let conn_info = LibraryConnectionInfo {
            connection: LibraryConnection::Internal {
                path: library_dir.to_owned(),
            },
            library_info,
            permissions: AccessRules::AllowByDefault { denylist: Vec::new() },
        };
        let prev_conn_info = connections.insert(domain.clone(), conn_info.clone());
        if let Some(prev_conn_info) = prev_conn_info {
            error!(
                "library domain name collision ({domain}): trying to insert {:?} but {:?} was already inserted",
                prev_conn_info, conn_info
            )
        }
    }
    connections
}

#[actix_web::main]
pub async fn server_main() -> std::io::Result<()> {
    log_fn_name!("web_main");
    const HOST: &str = "127.0.0.1";
    const PORT: u16 = 8080;
    info!("starting server on: http://{HOST}:{PORT}");

    static SERVER_CONFIG: LazyLock<ServerConfig> = LazyLock::new(ServerConfig::load);
    let local_library_connections: Arc<Mutex<LibraryConnections>> =
        Arc::new(Mutex::new(connect_local_libraries(&SERVER_CONFIG.internal_library_dirs)));

    HttpServer::new(move || {
        App::new()
            .app_data(AppData {
                config: Config::load().expect("todo"),
                server_config: &SERVER_CONFIG,
                connected_libraries: Arc::clone(&local_library_connections),
            })
            .service(index)
            .service(static_handler)
            .service(test::echo)
            .service(test::hey)
            .service(api::get_match_list)
            .service(api::get_match)
            .service(api::put_match)
    })
    .bind((HOST, PORT))?
    .run()
    .await
}
